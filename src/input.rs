// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{vec2, Vector2D};
use glutin::dpi::PhysicalPosition;
use glutin::event::{ModifiersState, MouseScrollDelta};

use crate::common::PixelSize;

use super::BedHandle;

pub(crate) struct InputState {
    scroll_delta: Vector2D<f32, PixelSize>,
    bed_handle: BedHandle,
    modifiers: ModifiersState,
}

impl InputState {
    pub(crate) fn new(bed_handle: BedHandle) -> InputState {
        InputState {
            scroll_delta: vec2(0.0, 0.0),
            bed_handle,
            modifiers: ModifiersState::empty(),
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
            10.0 * self.scroll_delta.x * self.scroll_delta.x.abs(),
            10.0 * self.scroll_delta.y * self.scroll_delta.y.abs(),
        );
        self.bed_handle.scroll_active_view(scroll_delta);
        self.scroll_delta = vec2(0.0, 0.0);
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
}
