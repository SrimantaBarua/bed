// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::Rect;

use crate::common::PixelSize;

pub(super) struct Cursor {
    pub(super) line_num: usize,
    pub(super) line_coff: usize,
}

impl Cursor {
    fn default() -> Cursor {
        Cursor {
            line_num: 0,
            line_coff: 0,
        }
    }
}

pub(super) struct BufferView {
    pub(super) rect: Rect<u32, PixelSize>,
    pub(super) cursors: Vec<Cursor>,
}

impl BufferView {
    pub(super) fn new(rect: Rect<u32, PixelSize>) -> BufferView {
        BufferView {
            rect: rect,
            cursors: vec![Cursor::default()],
        }
    }
}
