use std::cell::RefCell;
use std::io::{Read, Result as IOResult};
use std::rc::Rc;

use super::inner::BufferInner;
use super::view::BufferView;

pub struct Buffer {
    inner: Rc<RefCell<BufferInner>>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            inner: Rc::new(RefCell::new(BufferInner::new())),
        }
    }

    pub fn from_reader<R: Read>(reader: R) -> IOResult<Buffer> {
        BufferInner::from_reader(reader).map(|inner| Buffer {
            inner: Rc::new(RefCell::new(inner)),
        })
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    pub fn new_view(&self) -> BufferView {
        let view_id = self.inner.borrow_mut().create_view();
        BufferView::new(view_id, self.inner.clone())
    }
}
