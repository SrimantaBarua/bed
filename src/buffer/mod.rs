// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use crate::style::TextSize;
use crate::text::FontCollectionHandle;

use super::{Bed, BedHandle};

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
pub(crate) struct BufferBedHandle(Rc<RefCell<Bed>>);

impl BufferBedHandle {
    pub(crate) fn new(bed_handle: &BedHandle) -> BufferBedHandle {
        BufferBedHandle(bed_handle.0.clone())
    }

    fn text_font(&self) -> FontCollectionHandle {
        let inner = &*self.0.borrow();
        inner.text_font.clone()
    }

    fn text_size(&self) -> TextSize {
        let inner = &*self.0.borrow();
        inner.text_size
    }
}
