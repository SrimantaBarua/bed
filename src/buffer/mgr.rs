// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::io::Result as IOResult;
use std::rc::{Rc, Weak};

use fnv::FnvHashMap;

use crate::ts::TsCore;

use super::buffer::Buffer;
use super::{BufferID, BufferViewID};

pub(crate) struct BufferMgr {
    buffers: FnvHashMap<String, Weak<RefCell<Buffer>>>,
    next_view_id: usize,
    next_buf_id: usize,
    ts_core: TsCore,
}

// TODO: Periodically clear out Weak buffers with a strong count of 0

impl BufferMgr {
    pub(crate) fn new(ts_core: TsCore) -> BufferMgr {
        BufferMgr {
            buffers: FnvHashMap::default(),
            next_view_id: 0,
            next_buf_id: 0,
            ts_core,
        }
    }

    pub(crate) fn empty(&mut self) -> Rc<RefCell<Buffer>> {
        let ret = Rc::new(RefCell::new(Buffer::empty(BufferID(self.next_buf_id))));
        self.next_buf_id += 1;
        ret
    }

    pub(crate) fn from_file(&mut self, path: &str) -> IOResult<Rc<RefCell<Buffer>>> {
        self.buffers
            .get_mut(path)
            .and_then(|weak_ref| weak_ref.upgrade())
            .map(|buffer| {
                (&mut *buffer.borrow_mut())
                    .reload_from_file(path, &self.ts_core)
                    .map(|_| buffer.clone())
            })
            .unwrap_or_else(|| {
                Buffer::from_file(BufferID(self.next_buf_id), path, &self.ts_core).map(|buffer| {
                    self.next_buf_id += 1;
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
