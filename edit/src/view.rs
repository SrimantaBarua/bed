use std::cell::RefCell;
use std::ops::RangeBounds;
use std::rc::Rc;

use super::inner::BufferInner;
use super::point::Point;

pub struct BufferView {
    view_id: usize,
    buffer_inner: Rc<RefCell<BufferInner>>,
}

impl BufferView {
    pub fn contains_point(&self, point: &Point) -> bool {
        self.buffer_inner.borrow().contains_point(point)
    }

    pub fn insert_string(&mut self, point: &Point, s: &str) {
        self.buffer_inner.borrow_mut().insert_string(point, s)
    }

    pub fn remove<R>(&mut self, range: R)
    where
        R: RangeBounds<Point>,
    {
        self.buffer_inner.borrow_mut().remove(range)
    }

    pub(crate) fn new(view_id: usize, buffer_inner: Rc<RefCell<BufferInner>>) -> BufferView {
        BufferView {
            view_id,
            buffer_inner,
        }
    }
}

impl std::hash::Hash for BufferView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.view_id.hash(state);
        self.buffer_inner.as_ptr().hash(state);
    }
}

impl PartialEq for BufferView {
    fn eq(&self, other: &BufferView) -> bool {
        self.view_id == other.view_id && Rc::ptr_eq(&self.buffer_inner, &other.buffer_inner)
    }
}

impl Eq for BufferView {}

impl Drop for BufferView {
    fn drop(&mut self) {
        self.buffer_inner.borrow_mut().delete_view(self.view_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Buffer;

    #[test]
    fn view_ids() {
        let buffer = Buffer::new();
        let view0 = buffer.new_view();
        let view1 = buffer.new_view();
        assert!(Rc::ptr_eq(&view0.buffer_inner, &view1.buffer_inner));
        assert_eq!(view0.view_id, 0);
        assert_eq!(view1.view_id, 1);
    }
}