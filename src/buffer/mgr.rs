// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::io::Result as IOResult;
use std::rc::{Rc, Weak};

use fnv::FnvHashMap;

use crate::theme::Theme;
use crate::ts::TsCore;

use super::buffer::Buffer;
use super::{BufferID, BufferViewID};

pub(crate) struct BufferMgr {
    path_id_map: FnvHashMap<String, BufferID>,
    id_path_map: FnvHashMap<BufferID, String>,
    id_buf_map: FnvHashMap<BufferID, Weak<RefCell<Buffer>>>,
    next_view_id: usize,
    next_buf_id: usize,
    ts_core: TsCore,
    theme: Rc<Theme>,
}

// TODO: Periodically clear out Weak buffers with a strong count of 0

impl BufferMgr {
    pub(crate) fn new(ts_core: TsCore, theme: Rc<Theme>) -> BufferMgr {
        BufferMgr {
            path_id_map: FnvHashMap::default(),
            id_path_map: FnvHashMap::default(),
            id_buf_map: FnvHashMap::default(),
            next_view_id: 0,
            next_buf_id: 0,
            ts_core,
            theme,
        }
    }

    pub(crate) fn empty(&mut self) -> Rc<RefCell<Buffer>> {
        let buf_id = BufferID(self.next_buf_id);
        self.next_buf_id += 1;
        let ret = Rc::new(RefCell::new(Buffer::empty(buf_id, self.theme.clone())));
        self.id_buf_map.insert(buf_id, Rc::downgrade(&ret));
        ret
    }

    pub(crate) fn from_file(&mut self, path: &str) -> IOResult<Rc<RefCell<Buffer>>> {
        self.path_id_map
            .get(path)
            .and_then(|buf_id| self.id_buf_map.get(buf_id))
            .and_then(|weak_ref| weak_ref.upgrade())
            .map(|buffer| {
                (&mut *buffer.borrow_mut())
                    .reload_from_file(path, &self.ts_core)
                    .map(|_| buffer.clone())
            })
            .unwrap_or_else(|| {
                let bid = if let Some(bid) = self.path_id_map.get(path) {
                    *bid
                } else {
                    let bid = BufferID(self.next_buf_id);
                    self.next_buf_id += 1;
                    bid
                };
                Buffer::from_file(bid, path, &self.ts_core, self.theme.clone()).map(|buffer| {
                    let buffer = Rc::new(RefCell::new(buffer));
                    self.path_id_map.insert(path.to_owned(), bid);
                    self.id_path_map.insert(bid, path.to_owned());
                    self.id_buf_map.insert(bid, Rc::downgrade(&buffer));
                    buffer
                })
            })
    }

    pub(crate) fn write_buffer(
        &mut self,
        id: BufferID,
        opth: Option<String>,
    ) -> Option<IOResult<usize>> {
        let path = if let Some(path) = opth {
            path
        } else {
            if let Some(path) = self.id_path_map.get(&id) {
                path.to_owned()
            } else {
                return None;
            }
        };
        if let Some(rcbuf) = self.id_buf_map.get_mut(&id).and_then(|wr| wr.upgrade()) {
            let buf = &mut *rcbuf.borrow_mut();
            Some(buf.write(&path, &self.ts_core).map(|nb| {
                if let Some(p) = self.id_path_map.get(&id) {
                    self.path_id_map.remove(p);
                }
                self.id_path_map.insert(id, path.to_owned());
                self.path_id_map.insert(path, id);
                nb
            }))
        } else {
            self.id_buf_map.remove(&id);
            if let Some(path) = self.id_path_map.remove(&id) {
                self.path_id_map.remove(&path);
            }
            None
        }
    }

    pub(crate) fn load_buffer(
        &mut self,
        id: BufferID,
        opth: Option<String>,
    ) -> Option<IOResult<Rc<RefCell<Buffer>>>> {
        if let Some(path) = opth {
            Some(self.from_file(&path))
        } else {
            if let Some(path) = self.id_path_map.get(&id) {
                let path = path.to_owned();
                Some(self.from_file(&path))
            } else {
                return None;
            }
        }
    }

    pub(crate) fn next_view_id(&mut self) -> BufferViewID {
        let ret = BufferViewID(self.next_view_id);
        self.next_view_id += 1;
        ret
    }
}
