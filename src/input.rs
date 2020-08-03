// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{point2, vec2, Point2D, Vector2D};
use glutin::dpi::PhysicalPosition;
use glutin::event::{
    ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode,
};

use crate::buffer::CursorStyle;
use crate::common::PixelSize;

use super::BedHandle;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

pub(crate) struct InputState {
    mode: Mode,
    scroll_delta: Vector2D<f32, PixelSize>,
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
            scroll_delta: vec2(0.0, 0.0),
            bed_handle,
            modifiers: ModifiersState::empty(),
            cursor_pos: point2(0.0, 0.0),
            cursor_click_pos: None,
        }
    }

    pub(crate) fn add_scroll_delta(&mut self, delta: MouseScrollDelta) {
        let scroll = match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                if self.modifiers.shift() {
                    vec2(-y, x)
                } else {
                    vec2(x, -y)
                }
            }
            MouseScrollDelta::PixelDelta(log) => {
                let phys: PhysicalPosition<f64> = log.to_physical(self.bed_handle.scale_factor());
                if self.modifiers.shift() {
                    vec2(phys.x, -phys.y).cast()
                } else {
                    vec2(-phys.y, phys.x).cast()
                }
            }
        };
        self.scroll_delta += scroll;
    }

    pub(crate) fn update_modifiers(&mut self, m: ModifiersState) {
        self.modifiers = m;
    }

    pub(crate) fn flush_events(&mut self) {
        // Scroll
        let scroll_delta = vec2(
            20.0 * self.scroll_delta.x * self.scroll_delta.x.abs(),
            20.0 * self.scroll_delta.y * self.scroll_delta.y.abs(),
        );
        self.bed_handle.scroll_active_view(scroll_delta);
        self.scroll_delta = vec2(0.0, 0.0);
    }

    pub(crate) fn handle_char(&mut self, c: char) {
        match self.mode {
            Mode::Normal => match c {
                // Basic movement
                'h' => self.bed_handle.move_cursor_left(1),
                'j' => self.bed_handle.move_cursor_down(1),
                'k' => self.bed_handle.move_cursor_up(1),
                'l' => self.bed_handle.move_cursor_right(1),
                // Entering insert mode
                'i' => {
                    self.mode = Mode::Insert;
                    self.bed_handle.set_cursor_style(CursorStyle::Line);
                }
                _ => {}
            },
            Mode::Insert => match c as u32 {
                8 => {
                    // Backspace
                    self.bed_handle.delete_left(1)
                }
                127 => {
                    // Delete
                    self.bed_handle.delete_right(1)
                }
                _ => self.bed_handle.insert_char(c),
            },
        }
    }

    pub(crate) fn handle_keypress(&mut self, input: KeyboardInput) {
        if input.state != ElementState::Pressed {
            return;
        }
        if let Some(vkey) = input.virtual_keycode {
            match self.mode {
                Mode::Normal => match vkey {
                    // Basic movement
                    VirtualKeyCode::Up => self.bed_handle.move_cursor_up(1),
                    VirtualKeyCode::Down => self.bed_handle.move_cursor_down(1),
                    VirtualKeyCode::Left => self.bed_handle.move_cursor_left(1),
                    VirtualKeyCode::Right => self.bed_handle.move_cursor_right(1),
                    _ => {}
                },
                Mode::Insert => match vkey {
                    // Basic movement
                    VirtualKeyCode::Up => self.bed_handle.move_cursor_up(1),
                    VirtualKeyCode::Down => self.bed_handle.move_cursor_down(1),
                    VirtualKeyCode::Left => self.bed_handle.move_cursor_left(1),
                    VirtualKeyCode::Right => self.bed_handle.move_cursor_right(1),
                    // Exiting insert mode
                    VirtualKeyCode::Escape => {
                        self.mode = Mode::Normal;
                        self.bed_handle.set_cursor_style(CursorStyle::Block);
                    }
                    _ => {}
                },
            }
        }
    }

    pub(crate) fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        match button {
            MouseButton::Left if state == ElementState::Pressed => {
                self.cursor_click_pos = Some(self.cursor_pos);
                self.bed_handle.move_cursor_to_point(self.cursor_pos);
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

impl BedHandle {
    fn scroll_active_view(&mut self, scroll: Vector2D<f32, PixelSize>) {
        if scroll.x == 0.0 && scroll.y == 0.0 {
            return;
        }
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.active_mut().scroll(scroll);
    }

    fn move_cursor_up(&mut self, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.active_mut().move_cursor_up(n);
    }

    fn move_cursor_down(&mut self, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.active_mut().move_cursor_down(n);
    }

    fn move_cursor_left(&mut self, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.active_mut().move_cursor_left(n);
    }

    fn move_cursor_right(&mut self, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.active_mut().move_cursor_right(n);
    }

    fn move_cursor_to_point(&mut self, point: Point2D<f32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.move_cursor_to_point(point);
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.active_mut().set_cursor_style(style);
    }

    fn insert_char(&mut self, c: char) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.active_mut().insert_char(c);
    }

    fn delete_left(&mut self, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.active_mut().delete_left(n);
    }

    fn delete_right(&mut self, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_tree.active_mut().delete_right(n);
    }
}
