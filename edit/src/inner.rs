use std::io::{Read, Result as IOResult};

use ds::hash::FnvHashMap;
use rope::Rope;

use super::view_state::ViewState;

pub(crate) struct BufferInner {
    rope: Rope,
    view_id: usize,
    views: FnvHashMap<usize, ViewState>,
}

impl BufferInner {
    pub(crate) fn new() -> BufferInner {
        BufferInner {
            rope: Rope::new(),
            view_id: 0,
            views: FnvHashMap::default(),
        }
    }

    pub(crate) fn from_reader<R: Read>(reader: R) -> IOResult<BufferInner> {
        Rope::from_reader(reader).map(|rope| BufferInner {
            rope,
            view_id: 0,
            views: FnvHashMap::default(),
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.rope.len()
    }

    pub(crate) fn create_view(&mut self) -> usize {
        let view_id = self.view_id;
        self.view_id += 1;
        self.views.insert(view_id, ViewState::new());
        view_id
    }

    pub(crate) fn delete_view(&mut self, view_id: usize) {
        self.views.remove(&view_id);
    }
}
