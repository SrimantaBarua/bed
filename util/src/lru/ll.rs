use std::fmt;
use std::ptr;

// A node in the linked list.
struct Node<T> {
    data: T,
    next: NodeLink<T>,
    prev: NodePtr<T>,
}

impl<T> Node<T> {
    fn new(data: T) -> Node<T> {
        Node {
            data,
            next: None,
            prev: NodePtr::null(),
        }
    }
}

// An owned link to a node in the list. This is used for `next` pointers, or for the `head` node
// of a linked list.
type NodeLink<T> = Option<Box<Node<T>>>;

// A non-owning raw pointer to a node in the linked list. This is used for `prev` links and the
// `tail` pointer of the linked list. A `null` pointer is treated like `Option::None`. This is
// inherently unsafe though, because what if the data this points to is freed? Thus the `lru`
// module needs to be careful in how it uses this.
pub(super) struct NodePtr<T>(*mut Node<T>);

impl<T> Clone for NodePtr<T> {
    fn clone(&self) -> NodePtr<T> {
        NodePtr(self.0)
    }
}

impl<T> fmt::Debug for NodePtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NodePtr({:p})", self.0)
    }
}

impl<T> PartialEq for NodePtr<T> {
    fn eq(&self, other: &NodePtr<T>) -> bool {
        self.0 == other.0
    }
}

impl<T> Copy for NodePtr<T> {}
impl<T> Eq for NodePtr<T> {}

impl<T> NodePtr<T> {
    pub(super) unsafe fn data(&self) -> &T {
        &self.as_ref().unwrap().data
    }

    pub(super) unsafe fn data_mut(&mut self) -> &mut T {
        &mut self.as_mut().unwrap().data
    }

    fn null() -> NodePtr<T> {
        NodePtr(ptr::null_mut())
    }

    fn pointing_to(node: &mut Node<T>) -> NodePtr<T> {
        NodePtr(node)
    }

    fn ptr_eq(&self, boxed: &NodeLink<T>) -> bool {
        if let Some(data) = &boxed {
            self.0 == data.as_ref() as *const _ as *mut _
        } else {
            self.0.is_null()
        }
    }

    unsafe fn as_ref(&self) -> Option<&Node<T>> {
        if self.0.is_null() {
            None
        } else {
            Some(&*self.0)
        }
    }

    unsafe fn as_mut(&self) -> Option<&mut Node<T>> {
        if self.0.is_null() {
            None
        } else {
            Some(&mut *self.0)
        }
    }
}

pub(super) struct List<T> {
    head: NodeLink<T>,
    tail: NodePtr<T>,
}

impl<T> fmt::Debug for List<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.iter().collect::<Vec<_>>().fmt(f)
    }
}

impl<T> List<T> {
    // Creates a new empty linked list.
    pub(super) fn new() -> List<T> {
        List {
            head: None,
            tail: NodePtr::null(),
        }
    }

    // Pushes data to the end of the linked list, and returns a NodePtr pointing to the node which
    // contains the newly pushed data.
    pub(super) fn push_back(&mut self, data: T) -> NodePtr<T> {
        let mut new_node = Box::new(Node::new(data));
        let node_ptr = NodePtr::pointing_to(&mut new_node);
        self.link_at_end(new_node);
        node_ptr
    }

    // Deletes the front node of the list. If the list wasn't empty, return the data contained
    // within the front node.
    pub(super) fn pop_front(&mut self) -> Option<T> {
        if let Some(mut head) = self.head.take() {
            match head.next.take() {
                Some(mut next) => {
                    next.prev = NodePtr::null();
                    self.head = Some(next);
                }
                None => self.tail = NodePtr::null(),
            }
            Some(head.data)
        } else {
            None
        }
    }

    // Delete the provided node from the list. Now, this is unsafe, because the pointer might
    // point to data which has already been freed.
    pub(super) unsafe fn remove(&mut self, node_ptr: NodePtr<T>) -> Option<T> {
        if node_ptr.0.is_null() {
            None
        } else {
            Some(self.unlink(node_ptr).data)
        }
    }

    pub(super) unsafe fn move_to_end(&mut self, node_ptr: NodePtr<T>) {
        let node = self.unlink(node_ptr);
        self.link_at_end(node);
    }

    fn link_at_end(&mut self, mut node: Box<Node<T>>) {
        let node_ptr = NodePtr::pointing_to(&mut node);
        if let Some(tail) = unsafe { self.tail.as_mut() } {
            node.prev = self.tail;
            tail.next = Some(node);
        } else {
            self.head = Some(node);
        }
        self.tail = node_ptr;
    }

    unsafe fn unlink(&mut self, node_ptr: NodePtr<T>) -> Box<Node<T>> {
        let node = node_ptr.as_mut().unwrap();
        if let Some(next) = &mut node.next {
            assert_eq!(node_ptr, next.prev);
            next.prev = node.prev;
        } else {
            assert_eq!(node_ptr, self.tail);
            self.tail = node.prev;
        }
        if let Some(prev) = node.prev.as_mut() {
            assert!(node_ptr.ptr_eq(&prev.next));
            let mut node = prev.next.take().unwrap();
            prev.next = node.next.take();
            node
        } else {
            assert!(node_ptr.ptr_eq(&self.head));
            let mut node = self.head.take().unwrap();
            self.head = node.next.take();
            node
        }
    }

    fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            cur_node: self.head.as_deref(),
        }
    }
}

struct Iter<'a, T> {
    cur_node: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.cur_node.take().map(|cur| {
            self.cur_node = cur.next.as_deref();
            &cur.data
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Data<'a> {
        dropped: &'a mut bool,
    }

    impl<'a> Data<'a> {
        fn new(dropped: &'a mut bool) -> Data {
            Data { dropped }
        }
    }

    impl<'a> Drop for Data<'a> {
        fn drop(&mut self) {
            assert!(!*self.dropped);
            *self.dropped = true;
        }
    }

    #[test]
    fn list_operations() {
        let mut list = List::new();
        list.push_back(1);
        let ptr = list.push_back(2);
        list.push_back(3);
        assert_eq!(list.head.as_ref().unwrap().data, 1);
        assert_eq!(list.head.as_ref().unwrap().next.as_ref().unwrap().data, 2);
        assert_eq!(unsafe { list.tail.as_ref() }.unwrap().data, 3);
        assert_eq!(unsafe { list.remove(ptr) }, Some(2));
        assert_eq!(list.head.as_ref().unwrap().data, 1);
        assert_eq!(list.head.as_ref().unwrap().next.as_ref().unwrap().data, 3);
        assert_eq!(unsafe { list.tail.as_ref() }.unwrap().data, 3);
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.head.as_ref().unwrap().data, 3);
        assert_eq!(unsafe { list.tail.as_ref() }.unwrap().data, 3);
        assert_eq!(list.pop_front(), Some(3));
        assert!(list.head.is_none());
        assert!(list.pop_front().is_none());
        list.push_back(4);
        assert_eq!(list.head.as_ref().unwrap().data, 4);
        assert_eq!(unsafe { list.tail.as_ref() }.unwrap().data, 4);
        assert_eq!(list.pop_front(), Some(4));
        assert!(list.pop_front().is_none());
    }

    #[test]
    fn drop_count() {
        let mut dropped = false;
        {
            let mut list = List::new();
            let ptr = list.push_back(Data::new(&mut dropped));
            unsafe { list.remove(ptr) };
        }
        assert!(dropped);
    }

    #[test]
    fn drop_list() {
        let mut dropped = false;
        {
            let mut list = List::new();
            list.push_back(Data::new(&mut dropped));
        }
        assert!(dropped);
    }

    #[test]
    fn move_to_end() {
        let mut list = List::new();
        let p1 = list.push_back(1);
        assert_eq!(list.iter().collect::<Vec<_>>(), [&1]);
        unsafe { list.move_to_end(p1) };
        assert_eq!(list.iter().collect::<Vec<_>>(), [&1]);
        let p2 = list.push_back(2);
        assert_eq!(list.iter().collect::<Vec<_>>(), [&1, &2]);
        unsafe { list.move_to_end(p2) };
        assert_eq!(list.iter().collect::<Vec<_>>(), [&1, &2]);
        unsafe { list.move_to_end(p1) };
        assert_eq!(list.iter().collect::<Vec<_>>(), [&2, &1]);
        let p3 = list.push_back(3);
        assert_eq!(list.iter().collect::<Vec<_>>(), [&2, &1, &3]);
        unsafe { list.move_to_end(p2) };
        assert_eq!(list.iter().collect::<Vec<_>>(), [&1, &3, &2]);
        unsafe { list.move_to_end(p3) };
        assert_eq!(list.iter().collect::<Vec<_>>(), [&1, &2, &3]);
    }
}
