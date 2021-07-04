use std::ops::{Bound, RangeBounds};

use super::iter;
use super::rope::Rope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RopeSlice<'a> {
    pub(crate) rope: &'a Rope,
    pub(crate) start_offset: usize,
    pub(crate) end_offset: usize,
    pub(crate) newlines_before: usize,
    pub(crate) num_newlines: usize,
}

impl<'a> RopeSlice<'a> {
    pub fn slice<R: RangeBounds<usize>>(&self, range: R) -> RopeSlice<'a> {
        let start = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
        };
        let end = match range.end_bound() {
            Bound::Unbounded => self.len_bytes(),
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
        };
        assert!(start <= end, "slice start cannot be after end");
        assert!(end <= self.len_bytes(), "slice index out of bounds");
        let (start_offset, end_offset) = (self.start_offset + start, self.start_offset + end);
        let newlines_before = self.rope.root().num_newlines_upto(start_offset);
        let num_newlines = self.rope.root().num_newlines_upto(end_offset) - newlines_before;
        RopeSlice {
            rope: self.rope,
            start_offset: self.start_offset + start,
            end_offset: self.start_offset + end,
            newlines_before,
            num_newlines,
        }
    }

    pub fn to_string(&self) -> String {
        let mut ret = String::new();
        ret.reserve(self.len_bytes());
        for chunk in self.chunks() {
            ret.push_str(chunk);
        }
        ret
    }

    pub fn chunks(&self) -> iter::Chunks<'a> {
        iter::Chunks::new(self)
    }

    pub fn chars(&self) -> impl 'a + Iterator<Item = char> {
        self.chunks().flat_map(|s| s.chars())
    }

    pub fn char_indices(&self) -> iter::CharIndices<'a> {
        iter::CharIndices::new(self)
    }

    pub fn lines(&self) -> iter::Lines<'a> {
        iter::Lines::new(self)
    }

    pub fn len_bytes(&self) -> usize {
        self.end_offset - self.start_offset
    }

    pub fn len_chars(&self) -> usize {
        self.rope.root().num_chars_upto(self.end_offset)
            - self.rope.root().num_chars_upto(self.start_offset)
    }

    pub fn len_lines(&self) -> usize {
        self.num_newlines + 1
    }

    pub fn line(&self, index: usize) -> RopeSlice<'a> {
        assert!(index < self.len_lines(), "line index out of bounds");
        let start_offset = if index == 0 {
            self.start_offset
        } else {
            self.rope
                .root()
                .offset_for_newline(self.newlines_before + index - 1)
                + 1
        };
        let (num_newlines, end_offset) = if index == self.len_lines() - 1 {
            (0, self.end_offset)
        } else {
            (
                1,
                self.rope
                    .root()
                    .offset_for_newline(self.newlines_before + index)
                    + 1,
            )
        };
        RopeSlice {
            rope: self.rope,
            start_offset,
            end_offset,
            newlines_before: self.newlines_before + index,
            num_newlines,
        }
    }

    pub fn line_to_byte(&self, linum: usize) -> usize {
        if linum == 0 {
            0
        } else {
            self.rope
                .root()
                .offset_for_newline(self.newlines_before + linum - 1)
                + 1
                - self.start_offset
        }
    }

    pub fn byte_to_line(&self, index: usize) -> usize {
        self.rope
            .root()
            .num_newlines_upto(self.start_offset + index)
    }
}
