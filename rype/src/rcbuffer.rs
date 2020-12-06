// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::alloc::{alloc, dealloc, Layout};
use std::mem::{align_of, size_of};
use std::ops::{Deref, Range};
use std::ptr::copy_nonoverlapping;
use std::{fmt, slice};

/// Reference-counted dynamically-sized buffer of bytes
pub(crate) struct RcBuf {
    ptr: *mut RcBufInner, // Pointer to ref-counted part
    start: usize,         // Start offset of slice
    size: usize,          // End offset of slice
}

impl fmt::Debug for RcBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.header().data_slice().fmt(f)
    }
}

impl Clone for RcBuf {
    fn clone(&self) -> RcBuf {
        self.header_mut().strong_count += 1;
        RcBuf {
            ptr: self.ptr,
            start: self.start,
            size: self.size,
        }
    }
}

impl Drop for RcBuf {
    fn drop(&mut self) {
        let header = self.header_mut();
        header.strong_count -= 1;
        if header.strong_count == 0 {
            let data_size = header.size;
            let header_size = size_of::<RcBufInner>();
            let layout = Layout::from_size_align(header_size + data_size, align_of::<RcBufInner>())
                .expect("failed to create layout");
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl Deref for RcBuf {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.header().data_slice()[self.start..self.start + self.size]
    }
}

impl RcBuf {
    /// Wrap slice in RcBuf
    pub(crate) fn new<T: AsRef<[u8]>>(data: T) -> RcBuf {
        let data = data.as_ref();
        let data_size = data.len();
        assert!(data_size > 0, "empty slice");
        let header_size = size_of::<RcBufInner>();
        let layout = Layout::from_size_align(header_size + data_size, align_of::<RcBufInner>())
            .expect("failed to create layout");
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            panic!("failed to allocate buffer");
        }
        unsafe {
            let header = &mut *(ptr as *mut RcBufInner);
            header.size = data_size;
            header.strong_count = 1;
            let dst_ptr = header.data_ptr_mut();
            copy_nonoverlapping(data.as_ptr(), dst_ptr, data_size);
        };
        RcBuf {
            ptr: ptr as _,
            start: 0,
            size: data_size,
        }
    }

    /// Get a slice into this buffer - adds to the refcound. Panics if index out of bounds
    pub(crate) fn slice(&self, range: Range<usize>) -> RcBuf {
        assert!(!range.is_empty(), "empty range provided");
        assert!(range.end <= self.size);
        let mut ret = self.clone();
        ret.start = range.start;
        ret.size = range.end - range.start;
        ret
    }

    /// Get mutable reference to allocation header
    fn header(&self) -> &RcBufInner {
        unsafe { &*self.ptr }
    }

    /// Get const reference to allocation header
    fn header_mut(&self) -> &mut RcBufInner {
        unsafe { &mut *self.ptr }
    }
}

/// Allocation header
struct RcBufInner {
    size: usize,
    strong_count: usize,
}

impl RcBufInner {
    /// Get mut pointer to data
    fn data_ptr_mut(&mut self) -> *mut u8 {
        let ptr = self as *mut RcBufInner as *mut u8;
        unsafe { ptr.offset(size_of::<RcBufInner>() as isize) }
    }

    /// Get const pointer to data
    fn data_ptr(&self) -> *const u8 {
        let ptr = self as *const RcBufInner as *const u8;
        unsafe { ptr.offset(size_of::<RcBufInner>() as isize) }
    }

    /// Get slice of data
    fn data_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data_ptr(), self.size) }
    }
}

#[cfg(test)]
mod tests {
    use super::RcBuf;

    #[test]
    fn test_single() {
        let rc = RcBuf::new([1, 2, 3]);
        assert_eq!(rc.len(), 3);
        assert_eq!(rc.as_ref(), [1, 2, 3]);
    }

    #[test]
    fn test_cloned() {
        let rc = RcBuf::new([1, 2, 3]);
        {
            let cloned = rc.clone();
            assert_eq!(cloned.len(), 3);
            assert_eq!(cloned.as_ref(), [1, 2, 3]);
            assert_eq!(cloned.ptr, rc.ptr);
            assert_eq!(cloned.header().strong_count, 2);
        }
        assert_eq!(rc.header().strong_count, 1);
        assert_eq!(rc.len(), 3);
        assert_eq!(rc.as_ref(), [1, 2, 3]);
    }

    #[test]
    fn test_slice() {
        let rc = RcBuf::new([1, 2, 3, 4]);
        {
            let slice = rc.slice(1..3);
            assert_eq!(slice.len(), 2);
            assert_eq!(slice.as_ref(), [2, 3]);
            assert_eq!(slice.ptr, rc.ptr);
            assert_eq!(slice.header().strong_count, 2);
        }
        assert_eq!(rc.header().strong_count, 1);
        assert_eq!(rc.len(), 4);
        assert_eq!(rc.as_ref(), [1, 2, 3, 4]);
    }
}
