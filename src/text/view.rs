// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use crate::buffer::{Buffer, BufferViewID};

pub(crate) struct TextView {
    buffer: Rc<RefCell<Buffer>>,
    id: BufferViewID,
}

impl TextView {
    fn new(buffer: Rc<RefCell<Buffer>>, id: BufferViewID) -> TextView {
        TextView { buffer, id }
    }
}

pub(crate) struct TextPane {
    views: Vec<TextView>,
    active: usize,
}

impl TextPane {
    pub(super) fn new(buffer: Rc<RefCell<Buffer>>, view_id: BufferViewID) -> TextPane {
        let views = vec![TextView::new(buffer, view_id)];
        TextPane { views, active: 0 }
    }

    pub(super) fn clone(&self, view_id: BufferViewID) -> TextPane {
        let views = vec![TextView::new(
            self.views[self.active].buffer.clone(),
            view_id,
        )];
        TextPane { views, active: 0 }
    }
}
