// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::Rect;

use crate::common::PixelSize;

pub(super) struct BufferView {
    pub(super) rect: Rect<u32, PixelSize>,
}

impl BufferView {
    pub(super) fn new(rect: Rect<u32, PixelSize>) -> BufferView {
        BufferView { rect: rect }
    }
}
