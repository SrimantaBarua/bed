// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::Rect;

use crate::common::PixelSize;

pub(super) struct Cursor {
    pub(super) line_num: usize,
    pub(super) line_cidx: usize,
    pub(super) line_gidx: usize,
}

impl Cursor {
    fn default() -> Cursor {
        Cursor {
            line_num: 1,
            line_cidx: 2,
            line_gidx: 2,
        }
    }
}

pub(super) struct BufferView {
    pub(super) rect: Rect<u32, PixelSize>,
    pub(super) cursor: Cursor,
}

impl BufferView {
    pub(super) fn new(rect: Rect<u32, PixelSize>) -> BufferView {
        BufferView {
            rect: rect,
            cursor: Cursor::default(),
        }
    }
}
