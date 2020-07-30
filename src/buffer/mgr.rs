// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::io::Result as IOResult;

use fnv::FnvHashMap;

use super::{BufferBedHandle, BufferHandle, BufferViewId};

pub(crate) struct BufferMgr {
    bed_handle: BufferBedHandle,
    next_view_id: usize,
    path_buffer_map: FnvHashMap<String, BufferHandle>,
}

impl BufferMgr {
    pub(crate) fn new(bed_handle: BufferBedHandle) -> BufferMgr {
        BufferMgr {
            bed_handle,
            next_view_id: 0,
            path_buffer_map: FnvHashMap::default(),
        }
    }

    pub(crate) fn empty_buffer(&mut self) -> BufferHandle {
        BufferHandle::create_empty(self.bed_handle.clone())
    }

    pub(crate) fn read_file(&mut self, path: &str) -> IOResult<BufferHandle> {
        if let Some(buf_handle) = self.path_buffer_map.get(path) {
            let mut buf_handle = buf_handle.clone();
            buf_handle.reload_from_file(path)?;
            Ok(buf_handle)
        } else {
            let buffer = BufferHandle::create_from_file(path, self.bed_handle.clone())?;
            self.path_buffer_map.insert(path.to_owned(), buffer.clone());
            Ok(buffer)
        }
    }

    pub(crate) fn next_view_id(&mut self) -> BufferViewId {
        let ret = BufferViewId(self.next_view_id);
        self.next_view_id += 1;
        ret
    }
}
