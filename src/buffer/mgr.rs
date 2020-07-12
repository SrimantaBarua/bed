// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::io::Result as IOResult;
use std::rc::Rc;

use fnv::FnvHashMap;

use super::buffer::Buffer;

pub(crate) struct BufferMgr {
    path_buffer_map: FnvHashMap<String, Rc<RefCell<Buffer>>>,
}

impl BufferMgr {
    pub(crate) fn new() -> BufferMgr {
        BufferMgr {
            path_buffer_map: FnvHashMap::default(),
        }
    }

    pub(crate) fn empty_buffer(&mut self) -> Rc<RefCell<Buffer>> {
        Rc::new(RefCell::new(Buffer::empty()))
    }

    pub(crate) fn read_file(&mut self, path: &str) -> IOResult<Rc<RefCell<Buffer>>> {
        if let Some(ref_buffer) = self.path_buffer_map.get(path) {
            let ref_buffer = ref_buffer.clone();
            {
                let buffer = &mut *ref_buffer.borrow_mut();
                buffer.reload_from_file(path)?;
            }
            Ok(ref_buffer)
        } else {
            let buffer = Buffer::from_file(path)?;
            let buffer = Rc::new(RefCell::new(buffer));
            self.path_buffer_map.insert(path.to_owned(), buffer.clone());
            Ok(buffer)
        }
    }
}
