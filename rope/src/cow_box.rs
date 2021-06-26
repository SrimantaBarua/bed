use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// A copy-on-write wrapper around heap-allocated data. Cloning a `CowBox` is very cheap since it
/// just increments a refcount. Mutable access to a `CowBox` with refcount = 1 goes ahead as
/// normal. However if the refcount is greater than one, then it creates a new copy of the data.
pub(crate) struct CowBox<T: Clone>(NonNull<CowBoxInner<T>>);

impl<T: Clone> CowBox<T> {
    /// Creates a new `CowBox` wrapping the provided data.
    pub(crate) fn new(data: T) -> CowBox<T> {
        let inner = Box::new(CowBoxInner { refcount: 1, data });
        let inner_raw = Box::into_raw(inner);
        CowBox(unsafe { NonNull::new_unchecked(inner_raw) })
    }

    /// Gets an immutable reference to the wrapped data.
    fn data(&self) -> &T {
        unsafe { &self.0.as_ref().data }
    }

    /// Gets the refcount of the wrapped data. Primarily used for testing.
    #[allow(dead_code)]
    fn refcount(&self) -> usize {
        unsafe { self.0.as_ref().refcount }
    }
}

impl<T: Clone> Drop for CowBox<T> {
    /// Decrements the refcount for the wrapped data when a `CowBox` goes out of scope. When the
    /// refcount drops to 0, the wrapped data is deallocated.
    fn drop(&mut self) {
        let refcount = {
            let inner = unsafe { self.0.as_mut() };
            assert!(inner.refcount > 0);
            inner.refcount -= 1;
            inner.refcount
        };
        if refcount == 0 {
            unsafe { Box::from_raw(self.0.as_ptr()) };
        }
    }
}

impl<T: Clone> Clone for CowBox<T> {
    /// Increments the refcount for the wrapped data, and returns a `CowBox` which points to the
    /// same data.
    fn clone(&self) -> CowBox<T> {
        let inner_mut = unsafe { &mut *self.0.as_ptr() };
        inner_mut.refcount += 1;
        CowBox(self.0)
    }
}

impl<T: Clone> Deref for CowBox<T> {
    type Target = T;

    /// Returns an immutable reference to the wrapped data on immutably dereferencing a `CowBox`.
    fn deref(&self) -> &T {
        self.data()
    }
}

impl<T: Clone> DerefMut for CowBox<T> {
    /// Returns a mutable reference to the wrapped data on mutably dereferencing a `CowBox`. If the
    /// refcount of the wrapped data is greater than one, this creates a copy of the wrapped data
    /// and points the current `CowBox` to the copied data. All subsequent accesses includng the
    /// current one then happen on the newly copied data.
    fn deref_mut(&mut self) -> &mut T {
        let inner_mut = unsafe { self.0.as_mut() };
        assert!(inner_mut.refcount > 0);
        if inner_mut.refcount > 1 {
            inner_mut.refcount -= 1;
            let new_copy = Box::new(CowBoxInner {
                refcount: 1,
                data: inner_mut.data.clone(),
            });
            self.0 = unsafe { NonNull::new_unchecked(Box::into_raw(new_copy)) };
        }
        unsafe { &mut self.0.as_mut().data }
    }
}

impl<T: Clone + fmt::Debug> fmt::Debug for CowBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data().fmt(f)
    }
}

impl<T: Clone + PartialEq> PartialEq for CowBox<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.0 == other.0 {
            return true;
        }
        self.data().eq(other.data())
    }
}

impl<T: Clone + Eq> Eq for CowBox<T> {}

struct CowBoxInner<T: Clone> {
    refcount: usize,
    data: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refcounts() {
        let cow_box = CowBox::new(1);
        assert_eq!(cow_box.refcount(), 1);
        assert_eq!(*cow_box.data(), 1);
        {
            let cloned_box = cow_box.clone();
            assert_eq!(cow_box.refcount(), 2);
            assert_eq!(cloned_box.refcount(), 2);
            assert_eq!(*cow_box.data(), 1);
            assert_eq!(*cloned_box.data(), 1);
        }
        assert_eq!(cow_box.refcount(), 1);
        assert_eq!(*cow_box.data(), 1);
    }

    #[test]
    fn single_mutation() {
        let mut cow_box = CowBox::new(1);
        *cow_box = 2;
        assert_eq!(cow_box.refcount(), 1);
        assert_eq!(*cow_box.data(), 2);
    }

    #[test]
    fn copied_mutation() {
        let mut cow_box = CowBox::new(1);
        let mut cloned_box1 = cow_box.clone();
        assert_eq!(cow_box.refcount(), 2);
        assert_eq!(cloned_box1.refcount(), 2);
        let cloned_box2 = cow_box.clone();
        assert_eq!(cow_box.refcount(), 3);
        assert_eq!(cloned_box1.refcount(), 3);
        assert_eq!(cloned_box2.refcount(), 3);
        assert_eq!(*cow_box.data(), 1);
        assert_eq!(*cloned_box1.data(), 1);
        assert_eq!(*cloned_box2.data(), 1);
        *cow_box = 2;
        assert_eq!(cow_box.refcount(), 1);
        assert_eq!(cloned_box1.refcount(), 2);
        assert_eq!(cloned_box2.refcount(), 2);
        assert_eq!(*cow_box.data(), 2);
        assert_eq!(*cloned_box1.data(), 1);
        assert_eq!(*cloned_box2.data(), 1);
        *cloned_box1 = 3;
        assert_eq!(cow_box.refcount(), 1);
        assert_eq!(cloned_box1.refcount(), 1);
        assert_eq!(cloned_box2.refcount(), 1);
        assert_eq!(*cow_box.data(), 2);
        assert_eq!(*cloned_box1.data(), 3);
        assert_eq!(*cloned_box2.data(), 1);
        *cloned_box1 = 4;
        assert_eq!(cow_box.refcount(), 1);
        assert_eq!(cloned_box1.refcount(), 1);
        assert_eq!(cloned_box2.refcount(), 1);
        assert_eq!(*cow_box.data(), 2);
        assert_eq!(*cloned_box1.data(), 4);
        assert_eq!(*cloned_box2.data(), 1);
    }
}
