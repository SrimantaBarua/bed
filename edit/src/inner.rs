use std::io::{Read, Result as IOResult};

use rope::Rope;

pub(crate) struct BufferInner {
    rope: Rope,
    view_id: usize,
}

impl BufferInner {
    pub(crate) fn new() -> BufferInner {
        BufferInner {
            rope: Rope::new(),
            view_id: 0,
        }
    }

    pub(crate) fn from_reader<R: Read>(reader: R) -> IOResult<BufferInner> {
        Rope::from_reader(reader).map(|rope| BufferInner { rope, view_id: 0 })
    }

    pub(crate) fn len(&self) -> usize {
        self.rope.len()
    }

    pub(crate) fn next_view_id(&mut self) -> usize {
        let ret = self.view_id;
        self.view_id += 1;
        ret
    }
}
