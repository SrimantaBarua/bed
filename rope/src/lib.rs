use std::io::{Read, Result as IOResult};
use std::ops::{Bound, RangeBounds};

mod builder;
mod cow_box;

use cow_box::CowBox;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rope {
    root: CowBox<Node>,
}

impl Rope {
    pub fn new() -> Rope {
        Rope {
            root: CowBox::new(Node::new_leaf(LeafNode::new(String::new()))),
        }
    }

    pub fn from_reader<R: Read>(reader: R) -> IOResult<Rope> {
        builder::RopeBuilder::from_reader(reader).map(|builder| builder.build())
    }

    pub fn len(&self) -> usize {
        self.root.len()
    }

    pub fn slice<'a, R: RangeBounds<usize>>(&'a self, range: R) -> RopeSlice<'a> {
        self.whole_slice().slice(range)
    }

    pub fn to_string(&self) -> String {
        self.whole_slice().to_string()
    }

    fn whole_slice<'a>(&'a self) -> RopeSlice<'a> {
        RopeSlice {
            rope: self,
            start_offset: 0,
            end_offset: self.len(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RopeSlice<'a> {
    rope: &'a Rope,
    start_offset: usize,
    end_offset: usize,
}

impl<'a> RopeSlice<'a> {
    pub fn len(&self) -> usize {
        self.end_offset - self.start_offset
    }

    pub fn slice<R: RangeBounds<usize>>(&self, range: R) -> RopeSlice<'a> {
        let start = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
        };
        let end = match range.end_bound() {
            Bound::Unbounded => self.len(),
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
        };
        assert!(start <= end, "slice start cannot be after end");
        assert!(end <= self.len(), "slice index out of bounds");
        RopeSlice {
            rope: self.rope,
            start_offset: self.start_offset + start,
            end_offset: self.start_offset + end,
        }
    }

    pub fn to_string(&self) -> String {
        unimplemented!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Node {
    typ: NodeTyp,
}

impl Node {
    fn new_inner(inner_node: InnerNode) -> Node {
        Node {
            typ: NodeTyp::Inner(inner_node),
        }
    }

    fn new_leaf(leaf_node: LeafNode) -> Node {
        Node {
            typ: NodeTyp::Leaf(leaf_node),
        }
    }

    fn len(&self) -> usize {
        match &self.typ {
            NodeTyp::Inner(inner) => inner.len(),
            NodeTyp::Leaf(leaf) => leaf.len(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum NodeTyp {
    Inner(InnerNode),
    Leaf(LeafNode),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InnerNode {
    length: usize,
    left: CowBox<Node>,
    right: CowBox<Node>,
}

impl InnerNode {
    fn new(left: Node, right: Node) -> InnerNode {
        let length = left.len() + right.len();
        InnerNode {
            length,
            left: CowBox::new(left),
            right: CowBox::new(right),
        }
    }

    fn len(&self) -> usize {
        self.length
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LeafNode {
    data: String,
}

impl LeafNode {
    fn new(data: String) -> LeafNode {
        LeafNode { data }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

const MAX_NODE_SIZE: usize = 4096;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    fn open_file(path: &str) -> File {
        File::open(env!("CARGO_MANIFEST_DIR").to_owned() + path).unwrap()
    }

    #[test]
    fn len() {
        let rope = Rope::from_reader(open_file("/res/test1.txt")).unwrap();
        assert_eq!(rope.len(), 2412);
        let slice = rope.slice(2..);
        assert_eq!(slice.len(), 2410);
        assert_eq!(slice.start_offset, 2);
        assert_eq!(slice.end_offset, 2412);
        let slice = slice.slice(..2408);
        assert_eq!(slice.start_offset, 2);
        assert_eq!(slice.end_offset, 2410);
        assert_eq!(slice.len(), 2408);
        assert_eq!(slice.slice(..).len(), 2408);
        let slice = slice.slice(5..2400);
        assert_eq!(slice.len(), 2395);
        assert_eq!(slice.start_offset, 7);
        assert_eq!(slice.end_offset, 2402);
    }

    #[test]
    #[should_panic(expected = "slice index out of bounds")]
    fn slice_fail() {
        let rope = Rope::from_reader(open_file("/res/test1.txt")).unwrap();
        assert_eq!(rope.len(), 2412);
        rope.slice(..2413);
    }
}
