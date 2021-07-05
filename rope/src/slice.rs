use std::ops::{Bound, Range, RangeBounds};

use super::iter;
use super::rope::Rope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RopeSlice<'a> {
    pub(crate) rope: &'a Rope,
    pub(crate) start_bidx: usize,
    pub(crate) end_bidx: usize,
    newlines_before: usize,
    num_newlines: usize,
    chars_before: usize,
    num_chars: usize,
}

impl<'a> RopeSlice<'a> {
    pub fn slice<R: RangeBounds<usize>>(&self, char_range: R) -> RopeSlice<'a> {
        let start = match char_range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
        };
        let end = match char_range.end_bound() {
            Bound::Unbounded => self.len_chars(),
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
        };
        self.slice_bytes(self.char_to_byte(start)..self.char_to_byte(end))
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
        self.end_bidx - self.start_bidx
    }

    pub fn len_chars(&self) -> usize {
        self.num_chars
    }

    pub fn len_lines(&self) -> usize {
        self.num_newlines + 1
    }

    pub fn line(&self, index: usize) -> RopeSlice<'a> {
        assert!(index < self.len_lines(), "line index out of bounds");
        let start_offset = if index == 0 {
            0
        } else {
            self.rope
                .root()
                .bidx_for_newline(self.newlines_before + index - 1)
                + 1
                - self.start_bidx
        };
        let end_offset = if index == self.len_lines() - 1 {
            self.end_bidx - self.start_bidx
        } else {
            self.rope
                .root()
                .bidx_for_newline(self.newlines_before + index)
                + 1
                - self.start_bidx
        };
        self.slice_bytes(start_offset..end_offset)
    }

    pub fn line_to_byte(&self, linum: usize) -> usize {
        assert!(linum < self.len_lines(), "line index out of bounds");
        if linum == 0 {
            0
        } else {
            self.rope
                .root()
                .bidx_for_newline(self.newlines_before + linum - 1)
                + 1
                - self.start_bidx
        }
    }

    pub fn byte_to_line(&self, index: usize) -> usize {
        self.rope
            .root()
            .num_newlines_upto_bidx(self.start_bidx + index)
            - self.newlines_before
    }

    pub fn char_to_byte(&self, index: usize) -> usize {
        if index == self.len_chars() {
            return self.len_bytes();
        }
        self.rope.root().bidx_for_char(self.chars_before + index) - self.start_bidx
    }

    pub fn byte_to_char(&self, index: usize) -> usize {
        if index == self.len_bytes() {
            return self.len_chars();
        }
        self.rope
            .root()
            .num_chars_upto_bidx(self.start_bidx + index)
            - self.chars_before
    }

    pub fn line_to_char(&self, linum: usize) -> usize {
        self.byte_to_char(self.line_to_byte(linum))
    }

    pub fn char_to_line(&self, index: usize) -> usize {
        self.byte_to_line(self.char_to_byte(index))
    }

    pub(crate) fn slice_bytes(&self, byte_range: Range<usize>) -> RopeSlice<'a> {
        assert!(
            byte_range.start <= byte_range.end,
            "slice start cannot be after end"
        );
        assert!(
            byte_range.end <= self.len_bytes(),
            "slice index out of bounds"
        );
        let (start_bidx, end_bidx) = (
            self.start_bidx + byte_range.start,
            self.start_bidx + byte_range.end,
        );
        let newlines_before = self.rope.root().num_newlines_upto_bidx(start_bidx);
        let num_newlines = self.rope.root().num_newlines_upto_bidx(end_bidx) - newlines_before;
        let chars_before = self.rope.root().num_chars_upto_bidx(start_bidx);
        let num_chars = self.rope.root().num_chars_upto_bidx(end_bidx) - chars_before;
        RopeSlice {
            rope: self.rope,
            start_bidx,
            end_bidx,
            newlines_before,
            num_newlines,
            chars_before,
            num_chars,
        }
    }

    pub(crate) fn whole_slice(rope: &'a Rope) -> RopeSlice<'a> {
        RopeSlice {
            rope,
            start_bidx: 0,
            end_bidx: rope.len_bytes(),
            newlines_before: 0,
            num_newlines: rope.root().num_newlines(),
            chars_before: 0,
            num_chars: rope.len_chars(),
        }
    }
}
