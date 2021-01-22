// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefMut;
use std::ops::Drop;
use std::time::Duration;

use euclid::{point2, vec2, Point2D, Vector2D};
use glutin::dpi::PhysicalPosition;
use glutin::event::{
    ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode,
};

use crate::buffer::Mode as BufferMode;
use crate::common::PixelSize;
use crate::prompt::Prompt;
use crate::text::CursorStyle;
use crate::textview::{TextTree, TextViewEditCtx};
use crate::{Bed, BedHandle};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum MoveObj {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
    ToLine(usize),
    ToLastLine,
    ToViewFirstLine,
    ToViewLastLine,
    ToViewMiddleLine,
    LineStart(usize),
    LineEnd(usize),
    LineFirstNonBlank,
    WordBeg(usize, bool),
    WordEnd(usize, bool),
    Back(usize, bool),
}

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    Command,
    Change {
        action_mul: Option<usize>,
        move_mul: Option<usize>,
    },
    Delete {
        action_mul: Option<usize>,
        move_mul: Option<usize>,
    },
    Replace {
        action_mul: Option<usize>,
    },
    Normal {
        action_mul: Option<usize>,
    },
    Insert,
    PaneUpdate,
}

pub(crate) struct InputState {
    mode: Mode,
    scroll_amount: Vector2D<f32, PixelSize>,
    bed_handle: BedHandle,
    modifiers: ModifiersState,
    // Mouse
    cursor_pos: Point2D<f32, PixelSize>,
    cursor_click_pos: Option<Point2D<f32, PixelSize>>,
}

impl InputState {
    pub(crate) fn new(bed_handle: BedHandle) -> InputState {
        InputState {
            mode: Mode::Normal { action_mul: None },
            scroll_amount: vec2(0.0, 0.0),
            bed_handle,
            modifiers: ModifiersState::empty(),
            cursor_pos: point2(0.0, 0.0),
            cursor_click_pos: None,
        }
    }

    pub(crate) fn add_scroll_amount(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                self.scroll_amount += if self.modifiers.shift() {
                    vec2(-y, x) * 10.0
                } else {
                    vec2(x, -y) * 10.0
                };
            }
            MouseScrollDelta::PixelDelta(pp) => {
                self.scroll_amount += if self.modifiers.shift() {
                    vec2(-pp.y, pp.x).cast()
                } else {
                    vec2(pp.x, -pp.y).cast()
                };
            }
        }
    }

    pub(crate) fn update_modifiers(&mut self, m: ModifiersState) {
        self.modifiers = m;
    }

    pub(crate) fn flush_events(&mut self, duration: Duration) {
        // Scroll
        let mut acc = self.scroll_amount;
        if acc.x != 0.0 {
            acc.x *= acc.x.abs().sqrt()
        }
        if acc.y != 0.0 {
            acc.y *= acc.y.abs().sqrt()
        }
        self.bed_handle
            .edit()
            .scroll_views_with_active_acc(acc, duration);
        self.scroll_amount = vec2(0.0, 0.0);
    }

    pub(crate) fn handle_char(&mut self, c: char) {
        let mut bed = self.bed_handle.edit();
        let is_change_mode = match self.mode {
            Mode::Change { .. } => true,
            _ => false,
        };
        match &mut self.mode {
            Mode::Insert => match c as u32 {
                8 /* backspace */ => bed.edit_view().delete(MoveObj::Left(1)),
                127 /* delete */  => bed.edit_view().delete(MoveObj::Right(1)),
                _ => bed.edit_view().insert_char(c),
            },
            Mode::Normal { action_mul } => {
                let act_rep = action_mul.unwrap_or(1);
                let mut is_num = false;
                let mut next_mode = None;
                match c {
                    // Numbers
                    '1'..='9' => {
                        let num = (c as u32) - ('0' as u32);
                        *action_mul = match action_mul {
                            Some(x) => Some((*x * 10) + num as usize),
                            None => Some(num as usize),
                        };
                        is_num = true;
                    }
                    // Basic movement
                    'h' => bed.edit_view().move_cursor(MoveObj::Left(act_rep)),
                    'j' => bed.edit_view().move_cursor(MoveObj::Down(act_rep)),
                    'k' => bed.edit_view().move_cursor(MoveObj::Up(act_rep)),
                    'l' => bed.edit_view().move_cursor(MoveObj::Right(act_rep)),
                    // Move to line
                    'g' => {
                        let linum = action_mul.unwrap_or(1);
                        bed.edit_view().move_cursor(MoveObj::ToLine(linum - 1));
                    }
                    'G' => bed.edit_view().move_cursor(MoveObj::ToLastLine),
                    'H' => bed.edit_view().move_cursor(MoveObj::ToViewFirstLine),
                    'L' => bed.edit_view().move_cursor(MoveObj::ToViewLastLine),
                    'M' => bed.edit_view().move_cursor(MoveObj::ToViewMiddleLine),
                    // Object movement
                    'w' => bed
                        .edit_view()
                        .move_cursor(MoveObj::WordBeg(act_rep, false)),
                    'W' => bed.edit_view().move_cursor(MoveObj::WordBeg(act_rep, true)),
                    'e' => bed
                        .edit_view()
                        .move_cursor(MoveObj::WordEnd(act_rep, false)),
                    'E' => bed.edit_view().move_cursor(MoveObj::WordEnd(act_rep, true)),
                    'b' => bed
                        .edit_view()
                        .move_cursor(MoveObj::WordBeg(act_rep, false)),
                    'B' => bed.edit_view().move_cursor(MoveObj::WordBeg(act_rep, true)),
                    // Move to start/end of line
                    '0' => {
                        *action_mul = match action_mul {
                            Some(x) => {
                                is_num = true;
                                Some(*x * 10)
                            }
                            None => {
                                bed.edit_view().move_cursor(MoveObj::LineStart(1));
                                None
                            }
                        };
                    }
                    '$' => bed.edit_view().move_cursor(MoveObj::LineEnd(act_rep)),
                    '^' => bed.edit_view().move_cursor(MoveObj::LineFirstNonBlank),
                    // Entering insert mode
                    'i' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.set_buffer_mode(BufferMode::Insert);
                    }
                    'I' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor(MoveObj::LineStart(1));
                        ctx.set_buffer_mode(BufferMode::Insert);
                    }
                    'a' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor(MoveObj::Right(1));
                        ctx.set_buffer_mode(BufferMode::Insert);
                    }
                    'A' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor(MoveObj::LineEnd(1));
                    }
                    'o' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor(MoveObj::LineEnd(1));
                        ctx.insert_char('\n');
                        ctx.set_buffer_mode(BufferMode::Insert);
                    }
                    'O' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor(MoveObj::LineStart(1));
                        ctx.insert_char('\n');
                        ctx.move_cursor(MoveObj::Up(1));
                        ctx.set_buffer_mode(BufferMode::Insert);
                    }
                    's' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.delete(MoveObj::Right(act_rep));
                        ctx.set_buffer_mode(BufferMode::Insert);
                    }
                    'S' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor(MoveObj::LineStart(1));
                        ctx.delete(MoveObj::LineEnd(act_rep));
                        ctx.set_buffer_mode(BufferMode::Insert);
                    }
                    // Entering other modes
                    'c' => {
                        next_mode = Some(Mode::Change {
                            action_mul: *action_mul,
                            move_mul: None,
                        });
                        bed.edit_view().set_cursor_style(CursorStyle::Underline);
                    }
                    'd' => {
                        next_mode = Some(Mode::Delete {
                            action_mul: *action_mul,
                            move_mul: None,
                        });
                        bed.edit_view().set_cursor_style(CursorStyle::Underline);
                    }
                    'r' => {
                        next_mode = Some(Mode::Replace {
                            action_mul: *action_mul,
                        });
                        bed.edit_view().set_cursor_style(CursorStyle::Underline);
                    }
                    ':' => {
                        next_mode = Some(Mode::Command);
                        bed.edit_prompt().set_prompt(":");
                    }
                    // Delete
                    'x' => bed.edit_view().delete(MoveObj::Right(act_rep)),
                    _ => {}
                }
                if let Some(next) = next_mode {
                    self.mode = next;
                } else if !is_num {
                    *action_mul = None;
                }
            }
            Mode::Change {
                action_mul,
                move_mul,
            }
            | Mode::Delete {
                action_mul,
                move_mul,
            } => {
                let act_rep = action_mul.unwrap_or(1);
                let move_rep = move_mul.unwrap_or(1);
                let mut next_mode = Mode::Insert;
                let mut is_num = false;
                let mut ctx = bed.edit_view();
                match c {
                    // Numbers
                    '1'..='9' => {
                        let num = (c as u32) - ('0' as u32);
                        *move_mul = match move_mul {
                            Some(x) => Some((*x * 10) + num as usize),
                            None => Some(num as usize),
                        };
                        is_num = true;
                    }
                    // Basic movement
                    'h' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::Left(move_rep))
                        }
                    }
                    'j' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::Down(move_rep))
                        }
                    }
                    'k' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::Up(move_rep))
                        }
                    }
                    'l' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::Right(move_rep))
                        }
                    }
                    // Delete to line
                    'g' => {
                        let linum = action_mul.unwrap_or(1);
                        ctx.delete(MoveObj::ToLine(linum - 1));
                    }
                    'G' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::ToLastLine)
                        }
                    }
                    // Object
                    'c' if is_change_mode => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::Down(move_rep - 1))
                        }
                    }
                    'd' if !is_change_mode => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::Down(move_rep - 1))
                        }
                    }
                    // Object movement
                    'w' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::WordBeg(move_rep, false))
                        }
                    }
                    'W' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::WordBeg(move_rep, true))
                        }
                    }
                    'e' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::WordEnd(move_rep, false))
                        }
                    }
                    'E' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::WordEnd(move_rep, true))
                        }
                    }
                    'b' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::Back(move_rep, false))
                        }
                    }
                    'B' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::Back(move_rep, true))
                        }
                    }
                    // Delete to start/end of line
                    '0' => {
                        *move_mul = match move_mul {
                            Some(x) => {
                                is_num = true;
                                Some(*x * 10)
                            }
                            None => {
                                ctx.delete(MoveObj::LineStart(1));
                                None
                            }
                        };
                    }
                    '$' => {
                        for _ in 0..act_rep {
                            ctx.delete(MoveObj::LineEnd(move_rep))
                        }
                    }
                    '^' => ctx.delete(MoveObj::LineFirstNonBlank),
                    _ => next_mode = Mode::Normal { action_mul: None },
                }
                if !is_num {
                    if !is_change_mode {
                        next_mode = Mode::Normal { action_mul: None };
                    }
                    match next_mode {
                        Mode::Insert => {
                            ctx.set_cursor_style(CursorStyle::Line);
                            ctx.set_buffer_mode(BufferMode::Insert);
                        }
                        Mode::Normal { .. } => ctx.set_cursor_style(CursorStyle::Block),
                        _ => unreachable!(),
                    }
                    self.mode = next_mode;
                }
            }
            Mode::Replace { action_mul } => {
                let mut ctx = bed.edit_view();
                ctx.replace_repeated(c, action_mul.unwrap_or(1));
                ctx.set_cursor_style(CursorStyle::Block);
                self.mode = Mode::Normal { action_mul: None };
            }
            Mode::Command => {
                match c as u32 {
                    8 /* backspace */ => bed.edit_prompt().delete_left(),
                    127 /* delete */  => bed.edit_prompt().delete_right(),
                    10 | 13 /* newline / carriage return */ => {
                        if let Some(cmd) = bed.edit_prompt().get_command() {
                            bed.run_command(&cmd);
                            bed.edit_prompt().clear();
                        }
                        self.mode = Mode::Normal { action_mul: None };
                    }
                    _ => bed.edit_prompt().insert_char(c),
                }
            }
            Mode::PaneUpdate => {
                let tree = bed.edit_text_tree();
                let mut valid_input = true;
                match c {
                    'h' => tree.set_left_active(),
                    'H' => tree.move_left(),
                    'j' => tree.set_down_active(),
                    'J' => tree.move_down(),
                    'k' => tree.set_up_active(),
                    'K' => tree.move_up(),
                    'l' => tree.set_right_active(),
                    'L' => tree.move_right(),
                    '>' => tree.grow_active(),
                    '<' => tree.shrink_active(),
                    _ => valid_input = false,
                }
                if valid_input {
                    self.mode = Mode::Normal { action_mul: None };
                }
            }
        }
    }

    pub(crate) fn handle_keypress(&mut self, input: KeyboardInput) {
        let mut bed = self.bed_handle.edit();
        if input.state != ElementState::Pressed {
            return;
        }
        if let Some(vkey) = input.virtual_keycode {
            let is_change_mode = match self.mode {
                Mode::Change { .. } => true,
                _ => false,
            };
            // Handle mode-independent keys
            match vkey {
                VirtualKeyCode::Plus if self.modifiers.ctrl() => {
                    bed.edit_view().update_text_size(1);
                    return;
                }
                VirtualKeyCode::Minus if self.modifiers.ctrl() => {
                    bed.edit_view().update_text_size(-1);
                    return;
                }
                _ => {}
            }
            // Handle mode-specific keys
            match &mut self.mode {
                Mode::Insert => match vkey {
                    // Basic movement
                    VirtualKeyCode::Up => bed.edit_view().move_cursor(MoveObj::Up(1)),
                    VirtualKeyCode::Down => bed.edit_view().move_cursor(MoveObj::Down(1)),
                    VirtualKeyCode::Left => bed.edit_view().move_cursor(MoveObj::Left(1)),
                    VirtualKeyCode::Right => bed.edit_view().move_cursor(MoveObj::Right(1)),
                    VirtualKeyCode::Home => bed.edit_view().move_cursor(MoveObj::LineStart(1)),
                    VirtualKeyCode::End => bed.edit_view().move_cursor(MoveObj::LineEnd(1)),
                    // Exiting insert mode
                    VirtualKeyCode::Escape => {
                        self.mode = Mode::Normal { action_mul: None };
                        let mut ctx = bed.edit_view();
                        ctx.move_cursor(MoveObj::Left(1));
                        ctx.set_cursor_style(CursorStyle::Block);
                        ctx.set_buffer_mode(BufferMode::Normal);
                    }
                    _ => {}
                },
                Mode::Normal { action_mul } => {
                    let act_rep = action_mul.unwrap_or(1);
                    let mut reset_count = true;
                    let mut next_mode = None;
                    match vkey {
                        // Basic movement
                        VirtualKeyCode::Up => bed.edit_view().move_cursor(MoveObj::Up(act_rep)),
                        VirtualKeyCode::Down => bed.edit_view().move_cursor(MoveObj::Down(act_rep)),
                        VirtualKeyCode::Left => bed.edit_view().move_cursor(MoveObj::Left(act_rep)),
                        VirtualKeyCode::Right => {
                            bed.edit_view().move_cursor(MoveObj::Right(act_rep))
                        }
                        VirtualKeyCode::Home => bed.edit_view().move_cursor(MoveObj::LineStart(1)),
                        VirtualKeyCode::End => bed.edit_view().move_cursor(MoveObj::LineEnd(1)),
                        // Page up/down
                        VirtualKeyCode::F if self.modifiers == ModifiersState::CTRL => {
                            bed.edit_view().page_down()
                        }
                        VirtualKeyCode::B if self.modifiers == ModifiersState::CTRL => {
                            bed.edit_view().page_up()
                        }
                        VirtualKeyCode::D if self.modifiers == ModifiersState::CTRL => {
                            bed.edit_view().half_page_down()
                        }
                        VirtualKeyCode::U if self.modifiers == ModifiersState::CTRL => {
                            bed.edit_view().half_page_up()
                        }
                        // Pane update mode
                        VirtualKeyCode::W if self.modifiers == ModifiersState::CTRL => {
                            next_mode = Some(Mode::PaneUpdate)
                        }
                        _ => reset_count = false,
                    }
                    if reset_count {
                        *action_mul = None;
                    }
                    if let Some(mode) = next_mode {
                        self.mode = mode;
                    }
                }
                Mode::Delete {
                    action_mul,
                    move_mul,
                }
                | Mode::Change {
                    action_mul,
                    move_mul,
                } => {
                    let act_rep = action_mul.unwrap_or(1);
                    let move_rep = move_mul.unwrap_or(1);
                    let mut reset_mode = true;
                    let mut ctx = bed.edit_view();
                    for _ in 0..act_rep {
                        match vkey {
                            // Basic movement
                            VirtualKeyCode::Up => ctx.delete(MoveObj::Up(move_rep)),
                            VirtualKeyCode::Down => ctx.delete(MoveObj::Down(move_rep)),
                            VirtualKeyCode::Left => ctx.delete(MoveObj::Left(move_rep)),
                            VirtualKeyCode::Right => ctx.delete(MoveObj::Right(move_rep)),
                            VirtualKeyCode::Home => ctx.delete(MoveObj::LineStart(1)),
                            VirtualKeyCode::End => ctx.delete(MoveObj::LineEnd(1)),
                            _ => reset_mode = false,
                        }
                    }
                    if reset_mode {
                        if is_change_mode {
                            self.mode = Mode::Insert;
                            ctx.set_cursor_style(CursorStyle::Line);
                            ctx.set_buffer_mode(BufferMode::Normal);
                        } else {
                            self.mode = Mode::Normal { action_mul: None };
                            ctx.set_cursor_style(CursorStyle::Block);
                        }
                    }
                }
                Mode::Replace { .. } => match vkey {
                    VirtualKeyCode::Up
                    | VirtualKeyCode::Down
                    | VirtualKeyCode::Left
                    | VirtualKeyCode::Right
                    | VirtualKeyCode::Escape => {
                        self.mode = Mode::Normal { action_mul: None };
                        bed.edit_view().set_cursor_style(CursorStyle::Block);
                    }
                    _ => {}
                },
                Mode::Command => {
                    let cmd = bed.edit_prompt();
                    match vkey {
                        VirtualKeyCode::Left => cmd.move_left(),
                        VirtualKeyCode::Right => cmd.move_right(),
                        VirtualKeyCode::Home => cmd.move_start(),
                        VirtualKeyCode::End => cmd.move_end(),
                        VirtualKeyCode::Escape => {
                            cmd.clear();
                            self.mode = Mode::Normal { action_mul: None };
                        }
                        _ => {}
                    }
                }
                Mode::PaneUpdate => match vkey {
                    VirtualKeyCode::Escape => self.mode = Mode::Normal { action_mul: None },
                    VirtualKeyCode::W => {
                        if self.modifiers.shift() {
                            bed.edit_text_tree().cycle_prev();
                        } else {
                            bed.edit_text_tree().cycle_next();
                        }
                        self.mode = Mode::Normal { action_mul: None };
                    }
                    _ => {}
                },
            }
        }
    }

    pub(crate) fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        let mut bed = self.bed_handle.edit();
        match button {
            MouseButton::Left if state == ElementState::Pressed => {
                self.cursor_click_pos = Some(self.cursor_pos);
                bed.move_cursor_to_point(self.cursor_pos.cast());
            }
            MouseButton::Left if state == ElementState::Released => {
                self.cursor_click_pos = None;
            }
            _ => {}
        }
        self.mode = Mode::Normal { action_mul: None };
    }

    pub(crate) fn handle_cursor_moved(&mut self, phys_pos: PhysicalPosition<f64>) {
        self.cursor_pos = point2(phys_pos.x, phys_pos.y).cast();
    }
}

struct ViewEditCtx<'a> {
    view: TextViewEditCtx<'a>,
    update_global_x: bool,
}

impl<'a> Drop for ViewEditCtx<'a> {
    fn drop(&mut self) {
        self.view.snap_to_cursor(self.update_global_x);
    }
}

impl<'a> ViewEditCtx<'a> {
    fn move_cursor(&mut self, move_obj: MoveObj) {
        match move_obj {
            MoveObj::Up(_) | MoveObj::Down(_) => {}
            _ => self.update_global_x = true,
        }
        self.view.move_cursor(move_obj);
    }

    fn insert_char(&mut self, c: char) {
        self.view.insert_char(c);
        self.update_global_x = true;
    }

    fn delete(&mut self, move_obj: MoveObj) {
        self.view.delete(move_obj);
        self.update_global_x = true;
    }

    fn half_page_down(&mut self) {
        self.view.half_page_down();
        self.update_global_x = true;
    }

    fn half_page_up(&mut self) {
        self.view.half_page_up();
        self.update_global_x = true;
    }

    fn page_down(&mut self) {
        self.view.page_down();
        self.update_global_x = true;
    }

    fn page_up(&mut self) {
        self.view.page_up();
        self.update_global_x = true;
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        self.view.set_cursor_style(style);
        self.update_global_x = true;
    }

    fn replace_repeated(&mut self, c: char, n: usize) {
        self.view.replace_repeated(c, n);
        self.update_global_x = true;
    }

    fn update_text_size(&mut self, diff: i16) {
        self.view.update_text_size(diff);
    }

    fn set_buffer_mode(&mut self, mode: BufferMode) {
        self.view.set_buffer_mode(mode);
    }
}

struct BedEditCtx<'a> {
    bed: RefMut<'a, Bed>,
}

impl<'a> BedEditCtx<'a> {
    fn edit_view(&mut self) -> ViewEditCtx {
        ViewEditCtx {
            view: self.bed.text_tree.active_mut().edit_ctx(),
            update_global_x: false,
        }
    }

    fn edit_prompt(&mut self) -> &mut Prompt {
        &mut self.bed.prompt
    }

    fn edit_text_tree(&mut self) -> &mut TextTree {
        &mut self.bed.text_tree
    }

    fn scroll_views_with_active_acc(&mut self, acc: Vector2D<f32, PixelSize>, duration: Duration) {
        self.bed
            .text_tree
            .scroll_views_with_active_acc(acc, duration)
    }

    fn move_cursor_to_point(&mut self, point: Point2D<i32, PixelSize>) {
        self.bed.text_tree.move_cursor_to_point(point);
    }

    fn run_command(&mut self, cmd: &str) {
        self.bed.run_command(cmd)
    }
}

impl BedHandle {
    fn edit(&mut self) -> BedEditCtx {
        BedEditCtx {
            bed: self.0.borrow_mut(),
        }
    }
}
