// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::io::Result as IOResult;
use std::rc::{Rc, Weak};

use euclid::Size2D;
use fnv::FnvHashMap;

use crate::common::DPI;
use crate::font::FontCore;

use super::buffer::Buffer;
use super::BufferViewID;

pub(crate) struct BufferMgr {
    dpi: Size2D<u32, DPI>,
    font_core: Rc<RefCell<FontCore>>,
    buffers: FnvHashMap<String, Weak<RefCell<Buffer>>>,
    next_view_id: usize,
}

// TODO: Periodically clear out Weak buffers with a strong count of 0

impl BufferMgr {
    pub(crate) fn new(font_core: Rc<RefCell<FontCore>>, dpi: Size2D<u32, DPI>) -> BufferMgr {
        BufferMgr {
            dpi: dpi,
            font_core: font_core,
            buffers: FnvHashMap::default(),
            next_view_id: 0,
        }
    }

    pub(crate) fn empty(&mut self) -> Rc<RefCell<Buffer>> {
        Rc::new(RefCell::new(Buffer::empty(self.font_core.clone(), self.dpi)))
    }

    pub(crate) fn from_file(&mut self, path: &str) -> IOResult<Rc<RefCell<Buffer>>> {
        self.buffers
            .get_mut(path)
            .and_then(|weak_ref| weak_ref.upgrade())
            .map(|buffer| {
                (&mut *buffer.borrow_mut())
                    .reload_from_file(path)
                    .map(|_| buffer.clone())
            })
            .unwrap_or_else(|| {
                Buffer::from_file(path, self.font_core.clone(), self.dpi).map(|buffer| {
                    let buffer = Rc::new(RefCell::new(buffer));
                    self.buffers.insert(path.to_owned(), Rc::downgrade(&buffer));
                    buffer
                })
            })
    }

    pub(crate) fn next_view_id(&mut self) -> BufferViewID {
        let ret = BufferViewID(self.next_view_id);
        self.next_view_id += 1;
        ret
    }
}
