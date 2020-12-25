// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefMut;
use std::ops::Drop;
use std::time::Duration;

use euclid::{point2, vec2, Point2D, Vector2D};
use glutin::dpi::PhysicalPosition;
use glutin::event::{
    ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode,
};

use crate::buffer::CursorStyle;
use crate::common::PixelSize;
use crate::textview::TextViewEditCtx;
use crate::{Bed, BedHandle};

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    Normal { action_mul: Option<usize> },
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
        match &mut self.mode {
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
                        bed.edit_view().move_cursor_to_line(linum);
                    }
                    'G' => bed.edit_view().move_cursor_to_last_line(),
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
                    // Delete
                    'x' => bed.edit_view().delete_right(act_rep),
                    // Keep num chains continuing
                    _ => is_num = true,
                }
                if let Some(next) = next_mode {
                    self.mode = next;
                } else if !is_num {
                    *action_mul = None;
                }
            }
            Mode::Insert => match c as u32 {
                8 /* backspace */ => bed.edit_view().delete_left(1),
                127 /* delete */  => bed.edit_view().delete_right(1),
                _ => bed.edit_view().insert_char(c),
            },
        }
    }

    pub(crate) fn handle_keypress(&mut self, input: KeyboardInput) {
        let mut bed = self.bed_handle.edit();
        if input.state != ElementState::Pressed {
            return;
        }
        if let Some(vkey) = input.virtual_keycode {
            match &mut self.mode {
                Mode::Normal { action_mul } => {
                    let act_rep = action_mul.unwrap_or(1);
                    let mut reset_count = true;
                    match vkey {
                        // Basic movement
                        VirtualKeyCode::Up => bed.edit_view().move_cursor_up(act_rep),
                        VirtualKeyCode::Down => bed.edit_view().move_cursor_down(act_rep),
                        VirtualKeyCode::Left => bed.edit_view().move_cursor_left(act_rep),
                        VirtualKeyCode::Right => bed.edit_view().move_cursor_right(act_rep),
                        _ => reset_count = false,
                    }
                    if reset_count {
                        *action_mul = None;
                    }
                }
                Mode::Insert => match vkey {
                    // Basic movement
                    VirtualKeyCode::Up => bed.edit_view().move_cursor_up(1),
                    VirtualKeyCode::Down => bed.edit_view().move_cursor_down(1),
                    VirtualKeyCode::Left => bed.edit_view().move_cursor_left(1),
                    VirtualKeyCode::Right => bed.edit_view().move_cursor_right(1),
                    // Exiting insert mode
                    VirtualKeyCode::Escape => {
                        self.mode = Mode::Normal { action_mul: None };
                        let mut ctx = bed.edit_view();
                        ctx.move_cursor_left(1);
                        ctx.set_cursor_style(CursorStyle::Block);
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
    }

    pub(crate) fn handle_cursor_moved(&mut self, phys_pos: PhysicalPosition<f64>) {
        self.cursor_pos = point2(phys_pos.x, phys_pos.y).cast();
    }
}

struct ViewEditCtx<'a> {
    view: TextViewEditCtx<'a>,
}

impl<'a> Drop for ViewEditCtx<'a> {
    fn drop(&mut self) {
        self.view.snap_to_cursor();
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
    }

    fn move_cursor_right(&mut self, n: usize) {
        self.view.move_cursor_right(n);
    }

    fn move_cursor_to_line_start(&mut self, n: usize) {
        self.view.move_cursor_to_line_start(n);
    }

    fn move_cursor_to_line_end(&mut self, n: usize) {
        self.view.move_cursor_to_line_end(n);
    }

    fn move_cursor_to_line(&mut self, linum: usize) {
        self.view.move_cursor_to_line(linum);
    }

    fn move_cursor_to_last_line(&mut self) {
        self.view.move_cursor_to_last_line();
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        self.view.set_cursor_style(style);
    }

    fn insert_char(&mut self, c: char) {
        self.view.insert_char(c);
    }

    fn delete_left(&mut self, n: usize) {
        self.view.delete_left(n);
    }

    fn delete_right(&mut self, n: usize) {
        self.view.delete_right(n);
    }
}

struct BedEditCtx<'a> {
    bed: RefMut<'a, Bed>,
}

impl<'a> BedEditCtx<'a> {
    fn edit_view(&mut self) -> ViewEditCtx {
        ViewEditCtx {
            view: self.bed.text_tree.active_mut().edit_ctx(),
        }
    }

    fn scroll_views_with_active_acc(&mut self, acc: Vector2D<f32, PixelSize>, duration: Duration) {
        self.bed
            .text_tree
            .scroll_views_with_active_acc(acc, duration)
    }

    fn move_cursor_to_point(&mut self, point: Point2D<i32, PixelSize>) {
        self.bed.text_tree.move_cursor_to_point(point);
    }
}

impl BedHandle {
    fn edit(&mut self) -> BedEditCtx {
        BedEditCtx {
            bed: self.0.borrow_mut(),
        }
    }
}
