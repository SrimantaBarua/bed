// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use crate::style::TextSize;
use crate::text::FontCollectionHandle;

mod buffer;
mod mgr;
mod view;

pub(crate) use buffer::BufferHandle;
pub(crate) use mgr::BufferMgr;

// Handle to BufferView
#[derive(Eq, PartialEq, Hash)]
pub(crate) struct BufferViewId(usize);

impl BufferViewId {
    fn clone(&self) -> BufferViewId {
        BufferViewId(self.0)
    }
}

// Handle to editor state for buffer module
#[derive(Clone)]
pub(crate) struct BufferBedHandle(Rc<RefCell<BufferBedState>>);

impl BufferBedHandle {
    pub(crate) fn new(text_font: FontCollectionHandle, text_size: TextSize) -> BufferBedHandle {
        BufferBedHandle(Rc::new(RefCell::new(BufferBedState {
            needs_redraw: false,
            text_font,
            text_size,
        })))
    }

    pub(crate) fn set_text_font(&mut self, font: FontCollectionHandle) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_font = font;
    }

    pub(crate) fn set_text_size(&mut self, size: TextSize) {
        let inner = &mut *self.0.borrow_mut();
        inner.text_size = size;
    }

    pub(crate) fn text_font(&self) -> FontCollectionHandle {
        let inner = &*self.0.borrow();
        inner.text_font.clone()
    }

    pub(crate) fn text_size(&self) -> TextSize {
        let inner = &*self.0.borrow();
        inner.text_size
    }

    pub(crate) fn collect_redraw_state(&mut self) -> bool {
        let inner = &mut *self.0.borrow_mut();
        let ret = inner.needs_redraw;
        inner.needs_redraw = false;
        ret
    }

    fn request_redraw(&mut self) {
        let inner = &mut *self.0.borrow_mut();
        inner.needs_redraw = true;
    }
}

struct BufferBedState {
    needs_redraw: bool,
    text_font: FontCollectionHandle,
    text_size: TextSize,
}
