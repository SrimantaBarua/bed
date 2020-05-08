// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::io::Result as IOResult;
use std::rc::{Rc, Weak};

use fnv::FnvHashMap;

use super::buffer::Buffer;

pub(crate) struct BufferMgr {
    buffers: FnvHashMap<String, Weak<RefCell<Buffer>>>,
}

impl BufferMgr {
    pub(crate) fn new() -> BufferMgr {
        BufferMgr {
            buffers: FnvHashMap::default(),
        }
    }

    pub(crate) fn empty(&mut self) -> Rc<RefCell<Buffer>> {
        Rc::new(RefCell::new(Buffer::empty()))
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
                Buffer::from_file(path).map(|buffer| {
                    let buffer = Rc::new(RefCell::new(buffer));
                    self.buffers.insert(path.to_owned(), Rc::downgrade(&buffer));
                    buffer
                })
            })
    }
}
