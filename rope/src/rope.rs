use std::io::{Read, Result as IOResult};
use std::ops::{Bound, Range, RangeBounds};

use super::builder::RopeBuilder;
use super::cow_box::CowBox;
use super::iter;
use super::node::Node;
use super::slice::RopeSlice;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rope {
    root: CowBox<Node>,
}

impl Rope {
    pub fn new() -> Rope {
        Rope {
            root: CowBox::new(Node::new_leaf(String::new())),
        }
    }

    pub fn from_reader<R: Read>(reader: R) -> IOResult<Rope> {
        RopeBuilder::from_reader(reader).map(|builder| builder.build())
    }

    pub fn insert(&mut self, char_index: usize, data: &str) {
        self.root.insert(char_index, data);
    }

    pub fn insert_char(&mut self, char_index: usize, c: char) {
        let mut buf = [0; 6];
        self.insert(char_index, c.encode_utf8(&mut buf));
    }

    pub fn remove<R: RangeBounds<usize>>(&mut self, char_range: R) {
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
        assert!(start <= end, "start cannot be after end");
        assert!(end <= self.len_chars(), "index out of bounds");
        if start == 0 && end == self.len_chars() {
            self.root = CowBox::new(Node::new_leaf(String::new()));
        } else {
            self.root.remove(start..end);
        }
    }

    pub fn len_bytes(&self) -> usize {
        self.root.num_bytes()
    }

    pub fn len_chars(&self) -> usize {
        self.root.num_bytes()
    }

    pub fn len_lines(&self) -> usize {
        self.root.num_newlines() + 1
    }

    pub fn slice<'a, R: RangeBounds<usize>>(&'a self, char_range: R) -> RopeSlice<'a> {
        self.whole_slice().slice(char_range)
    }

    pub fn to_string(&self) -> String {
        self.whole_slice().to_string()
    }

    pub fn chunks<'a>(&'a self) -> iter::Chunks<'a> {
        self.whole_slice().chunks()
    }

    pub fn chars<'a>(&'a self) -> impl 'a + Iterator<Item = char> {
        self.whole_slice().chars()
    }

    pub fn char_indices<'a>(&'a self) -> iter::CharIndices<'a> {
        self.whole_slice().char_indices()
    }

    pub fn lines<'a>(&'a self) -> iter::Lines<'a> {
        self.whole_slice().lines()
    }

    pub fn line<'a>(&'a self, index: usize) -> RopeSlice<'a> {
        self.whole_slice().line(index)
    }

    pub fn line_to_byte(&self, linum: usize) -> usize {
        self.whole_slice().line_to_byte(linum)
    }

    pub fn byte_to_line(&self, index: usize) -> usize {
        self.whole_slice().byte_to_line(index)
    }

    pub fn char_to_byte(&self, index: usize) -> usize {
        self.whole_slice().char_to_byte(index)
    }

    pub fn byte_to_char(&self, index: usize) -> usize {
        self.whole_slice().byte_to_char(index)
    }

    pub(crate) fn root(&self) -> &Node {
        &*self.root
    }

    pub(crate) fn slice_bytes<'a>(&'a self, byte_range: Range<usize>) -> RopeSlice<'a> {
        self.whole_slice().slice_bytes(byte_range)
    }

    fn whole_slice<'a>(&'a self) -> RopeSlice<'a> {
        RopeSlice::whole_slice(self)
    }
}

impl From<Node> for Rope {
    fn from(node: Node) -> Rope {
        Rope {
            root: CowBox::new(node),
        }
    }
}
