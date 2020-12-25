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

#[derive(Clone, Copy, Eq, PartialEq)]
enum Mode {
    Normal,
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
            mode: Mode::Normal,
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
        match self.mode {
            Mode::Normal => match c {
                // Basic movement
                'h' => bed.edit_view().move_cursor_left(1),
                'j' => bed.edit_view().move_cursor_down(1),
                'k' => bed.edit_view().move_cursor_up(1),
                'l' => bed.edit_view().move_cursor_right(1),
                '0' => bed.edit_view().move_cursor_to_line_start(1),
                '$' => bed.edit_view().move_cursor_to_line_end(1),
                // Entering insert mode
                'i' => {
                    self.mode = Mode::Insert;
                    bed.edit_view().set_cursor_style(CursorStyle::Line)
                }
                'I' => {
                    self.mode = Mode::Insert;
                    let mut ctx = bed.edit_view();
                    ctx.set_cursor_style(CursorStyle::Line);
                    ctx.move_cursor_to_line_start(1);
                }
                'a' => {
                    self.mode = Mode::Insert;
                    let mut ctx = bed.edit_view();
                    ctx.set_cursor_style(CursorStyle::Line);
                    ctx.move_cursor_right(1);
                }
                'A' => {
                    self.mode = Mode::Insert;
                    let mut ctx = bed.edit_view();
                    ctx.set_cursor_style(CursorStyle::Line);
                    ctx.move_cursor_to_line_end(1);
                }
                // Delete
                'x' => bed.edit_view().delete_right(1),
                _ => {}
            },
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
            match self.mode {
                Mode::Normal => match vkey {
                    // Basic movement
                    VirtualKeyCode::Up => bed.edit_view().move_cursor_up(1),
                    VirtualKeyCode::Down => bed.edit_view().move_cursor_down(1),
                    VirtualKeyCode::Left => bed.edit_view().move_cursor_left(1),
                    VirtualKeyCode::Right => bed.edit_view().move_cursor_right(1),
                    _ => {}
                },
                Mode::Insert => match vkey {
                    // Basic movement
                    VirtualKeyCode::Up => bed.edit_view().move_cursor_up(1),
                    VirtualKeyCode::Down => bed.edit_view().move_cursor_down(1),
                    VirtualKeyCode::Left => bed.edit_view().move_cursor_left(1),
                    VirtualKeyCode::Right => bed.edit_view().move_cursor_right(1),
                    // Exiting insert mode
                    VirtualKeyCode::Escape => {
                        self.mode = Mode::Normal;
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
