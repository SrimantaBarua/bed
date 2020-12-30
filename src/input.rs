// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefMut;
use std::ops::Drop;
use std::time::Duration;

use euclid::{point2, vec2, Point2D, Vector2D};
use glutin::dpi::PhysicalPosition;
use glutin::event::{
    ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode,
};

use crate::common::PixelSize;
use crate::prompt::Prompt;
use crate::text::CursorStyle;
use crate::textview::TextViewEditCtx;
use crate::{Bed, BedHandle};

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
                8 /* backspace */ => bed.edit_view().delete_left(1),
                127 /* delete */  => bed.edit_view().delete_right(1),
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
                    'h' => bed.edit_view().move_cursor_left(act_rep),
                    'j' => bed.edit_view().move_cursor_down(act_rep),
                    'k' => bed.edit_view().move_cursor_up(act_rep),
                    'l' => bed.edit_view().move_cursor_right(act_rep),
                    // Move to line
                    'g' => {
                        let linum = action_mul.unwrap_or(1);
                        bed.edit_view().move_cursor_to_line(linum - 1);
                    }
                    'G' => bed.edit_view().move_cursor_to_last_line(),
                    // Object movement
                    'w' => bed.edit_view().move_word(act_rep),
                    'W' => bed.edit_view().move_word_extended(act_rep),
                    'e' => bed.edit_view().move_word_end(act_rep),
                    'E' => bed.edit_view().move_word_end_extended(act_rep),
                    'b' => bed.edit_view().move_back(act_rep),
                    'B' => bed.edit_view().move_back_extended(act_rep),
                    // Move to start/end of line
                    '0' => {
                        *action_mul = match action_mul {
                            Some(x) => {
                                is_num = true;
                                Some(*x * 10)
                            }
                            None => {
                                bed.edit_view().move_cursor_to_line_start(1);
                                None
                            }
                        };
                    }
                    '$' => bed.edit_view().move_cursor_to_line_end(act_rep),
                    // Entering insert mode
                    'i' => {
                        next_mode = Some(Mode::Insert);
                        bed.edit_view().set_cursor_style(CursorStyle::Line)
                    }
                    'I' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor_to_line_start(1);
                    }
                    'a' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor_right(1);
                    }
                    'A' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor_to_line_end(1);
                    }
                    'o' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor_to_line_end(1);
                        ctx.insert_char('\n');
                    }
                    'O' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor_to_line_start(1);
                        ctx.insert_char('\n');
                        ctx.move_cursor_up(1);
                    }
                    's' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.delete_right(act_rep);
                    }
                    'S' => {
                        next_mode = Some(Mode::Insert);
                        let mut ctx = bed.edit_view();
                        ctx.set_cursor_style(CursorStyle::Line);
                        ctx.move_cursor_to_line_start(1);
                        ctx.delete_to_line_end(act_rep);
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
                    'x' => bed.edit_view().delete_right(act_rep),
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
                            ctx.delete_left(move_rep)
                        }
                    }
                    'j' => {
                        for _ in 0..act_rep {
                            ctx.delete_down(move_rep)
                        }
                    }
                    'k' => {
                        for _ in 0..act_rep {
                            ctx.delete_up(move_rep)
                        }
                    }
                    'l' => {
                        for _ in 0..act_rep {
                            ctx.delete_right(move_rep)
                        }
                    }
                    // Delete to line
                    'g' => {
                        let linum = action_mul.unwrap_or(1);
                        ctx.delete_to_line(linum - 1);
                    }
                    'G' => {
                        for _ in 0..act_rep {
                            ctx.delete_to_last_line()
                        }
                    }
                    // Object
                    'c' if is_change_mode => {
                        for _ in 0..act_rep {
                            ctx.delete_down(move_rep - 1)
                        }
                    }
                    'd' if !is_change_mode => {
                        for _ in 0..act_rep {
                            ctx.delete_down(move_rep - 1)
                        }
                    }
                    // Object movement
                    'w' => {
                        for _ in 0..act_rep {
                            ctx.delete_word(move_rep)
                        }
                    }
                    'W' => {
                        for _ in 0..act_rep {
                            ctx.delete_word_extended(move_rep)
                        }
                    }
                    'e' => {
                        for _ in 0..act_rep {
                            ctx.delete_word_end(move_rep)
                        }
                    }
                    'E' => {
                        for _ in 0..act_rep {
                            ctx.delete_word_end_extended(move_rep)
                        }
                    }
                    'b' => {
                        for _ in 0..act_rep {
                            ctx.delete_back(move_rep)
                        }
                    }
                    'B' => {
                        for _ in 0..act_rep {
                            ctx.delete_back_extended(move_rep)
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
                                ctx.delete_to_line_start(1);
                                None
                            }
                        };
                    }
                    '$' => {
                        for _ in 0..act_rep {
                            ctx.delete_to_line_end(move_rep)
                        }
                    }
                    _ => next_mode = Mode::Normal { action_mul: None },
                }
                if !is_num {
                    if !is_change_mode {
                        next_mode = Mode::Normal { action_mul: None };
                    }
                    match next_mode {
                        Mode::Insert => ctx.set_cursor_style(CursorStyle::Line),
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
                    VirtualKeyCode::Up => bed.edit_view().move_cursor_up(1),
                    VirtualKeyCode::Down => bed.edit_view().move_cursor_down(1),
                    VirtualKeyCode::Left => bed.edit_view().move_cursor_left(1),
                    VirtualKeyCode::Right => bed.edit_view().move_cursor_right(1),
                    VirtualKeyCode::Home => bed.edit_view().move_cursor_to_line_start(1),
                    VirtualKeyCode::End => bed.edit_view().move_cursor_to_line_end(1),
                    // Exiting insert mode
                    VirtualKeyCode::Escape => {
                        self.mode = Mode::Normal { action_mul: None };
                        let mut ctx = bed.edit_view();
                        ctx.move_cursor_left(1);
                        ctx.set_cursor_style(CursorStyle::Block);
                    }
                    _ => {}
                },
                Mode::Normal { action_mul } => {
                    let act_rep = action_mul.unwrap_or(1);
                    let mut reset_count = true;
                    match vkey {
                        // Basic movement
                        VirtualKeyCode::Up => bed.edit_view().move_cursor_up(act_rep),
                        VirtualKeyCode::Down => bed.edit_view().move_cursor_down(act_rep),
                        VirtualKeyCode::Left => bed.edit_view().move_cursor_left(act_rep),
                        VirtualKeyCode::Right => bed.edit_view().move_cursor_right(act_rep),
                        VirtualKeyCode::Home => bed.edit_view().move_cursor_to_line_start(1),
                        VirtualKeyCode::End => bed.edit_view().move_cursor_to_line_end(1),
                        _ => reset_count = false,
                    }
                    if reset_count {
                        *action_mul = None;
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
                            VirtualKeyCode::Up => ctx.delete_up(move_rep),
                            VirtualKeyCode::Down => ctx.delete_down(move_rep),
                            VirtualKeyCode::Left => ctx.delete_left(move_rep),
                            VirtualKeyCode::Right => ctx.delete_right(move_rep),
                            VirtualKeyCode::Home => ctx.delete_to_line_start(1),
                            VirtualKeyCode::End => ctx.delete_to_line_end(1),
                            _ => reset_mode = false,
                        }
                    }
                    if reset_mode {
                        if is_change_mode {
                            self.mode = Mode::Insert;
                            ctx.set_cursor_style(CursorStyle::Line);
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
    fn move_cursor_up(&mut self, n: usize) {
        self.view.move_cursor_up(n);
    }

    fn move_cursor_down(&mut self, n: usize) {
        self.view.move_cursor_down(n);
    }

    fn move_cursor_left(&mut self, n: usize) {
        self.view.move_cursor_left(n);
        self.update_global_x = true;
    }

    fn move_cursor_right(&mut self, n: usize) {
        self.view.move_cursor_right(n);
        self.update_global_x = true;
    }

    fn move_cursor_to_line_start(&mut self, n: usize) {
        self.view.move_cursor_to_line_start(n);
        self.update_global_x = true;
    }

    fn move_cursor_to_line_end(&mut self, n: usize) {
        self.view.move_cursor_to_line_end(n);
        self.update_global_x = true;
    }

    fn move_cursor_to_line(&mut self, linum: usize) {
        self.view.move_cursor_to_line(linum);
        self.update_global_x = true;
    }

    fn move_cursor_to_last_line(&mut self) {
        self.view.move_cursor_to_last_line();
        self.update_global_x = true;
    }

    fn move_word(&mut self, n: usize) {
        self.view.move_cursor_word(n);
        self.update_global_x = true;
    }

    fn move_word_extended(&mut self, n: usize) {
        self.view.move_cursor_word_extended(n);
        self.update_global_x = true;
    }

    fn move_word_end(&mut self, n: usize) {
        self.view.move_cursor_word_end(n);
        self.update_global_x = true;
    }

    fn move_word_end_extended(&mut self, n: usize) {
        self.view.move_cursor_word_end_extended(n);
        self.update_global_x = true;
    }

    fn move_back(&mut self, n: usize) {
        self.view.move_cursor_back(n);
        self.update_global_x = true;
    }

    fn move_back_extended(&mut self, n: usize) {
        self.view.move_cursor_back_extended(n);
        self.update_global_x = true;
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        self.view.set_cursor_style(style);
        self.update_global_x = true;
    }

    fn insert_char(&mut self, c: char) {
        self.view.insert_char(c);
        self.update_global_x = true;
    }

    fn delete_left(&mut self, n: usize) {
        self.view.delete_left(n);
        self.update_global_x = true;
    }

    fn delete_right(&mut self, n: usize) {
        self.view.delete_right(n);
        self.update_global_x = true;
    }

    fn delete_up(&mut self, n: usize) {
        self.view.delete_up(n);
        self.update_global_x = true;
    }

    fn delete_down(&mut self, n: usize) {
        self.view.delete_down(n);
        self.update_global_x = true;
    }

    fn delete_to_line(&mut self, n: usize) {
        self.view.delete_to_line(n);
        self.update_global_x = true;
    }

    fn delete_to_last_line(&mut self) {
        self.view.delete_to_last_line();
        self.update_global_x = true;
    }

    fn delete_word(&mut self, n: usize) {
        self.view.delete_word(n);
        self.update_global_x = true;
    }

    fn delete_word_extended(&mut self, n: usize) {
        self.view.delete_word_extended(n);
        self.update_global_x = true;
    }

    fn delete_word_end(&mut self, n: usize) {
        self.view.delete_word_end(n);
        self.update_global_x = true;
    }

    fn delete_word_end_extended(&mut self, n: usize) {
        self.view.delete_word_end_extended(n);
        self.update_global_x = true;
    }

    fn delete_back(&mut self, n: usize) {
        self.view.delete_back(n);
        self.update_global_x = true;
    }

    fn delete_back_extended(&mut self, n: usize) {
        self.view.delete_back_extended(n);
        self.update_global_x = true;
    }

    fn delete_to_line_start(&mut self, n: usize) {
        self.view.delete_to_line_start(n);
        self.update_global_x = true;
    }

    fn delete_to_line_end(&mut self, n: usize) {
        self.view.delete_to_line_end(n);
        self.update_global_x = true;
    }

    fn replace_repeated(&mut self, c: char, n: usize) {
        self.view.replace_repeated(c, n);
        self.update_global_x = true;
    }

    fn update_text_size(&mut self, diff: i16) {
        self.view.update_text_size(diff);
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
