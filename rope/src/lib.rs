use std::io::{Read, Result as IOResult};
use std::ops::{Bound, RangeBounds};

mod builder;
mod cow_box;
mod take_mut;

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

    pub fn insert(&mut self, index: usize, data: &str) {
        self.root.insert(index, data);
    }

    pub fn insert_char(&mut self, index: usize, c: char) {
        let mut buf = [0; 6];
        self.insert(index, c.encode_utf8(&mut buf));
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

    pub fn chunks<'a>(&'a self) -> Chunks<'a> {
        self.whole_slice().chunks()
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
        let mut ret = String::new();
        ret.reserve(self.len());
        for chunk in self.chunks() {
            ret.push_str(chunk);
        }
        ret
    }

    pub fn chunks(&self) -> Chunks<'a> {
        Chunks::new(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunks<'a> {
    stack: Vec<&'a InnerNode>,
    next_leaf: Option<&'a LeafNode>,
    start_offset: usize,
    remaining: usize,
}

impl<'a> Chunks<'a> {
    fn new(rope_slice: &RopeSlice<'a>) -> Chunks<'a> {
        if rope_slice.len() == 0 {
            return Chunks {
                stack: vec![],
                next_leaf: None,
                start_offset: 0,
                remaining: 0,
            };
        }
        let mut stack = Vec::new();
        let mut start_offset = rope_slice.start_offset;
        let mut cur_node = &*rope_slice.rope.root;
        let next_leaf = loop {
            match &cur_node.typ {
                NodeTyp::Inner(inner) => {
                    let left_len = inner.left.len();
                    if start_offset > left_len {
                        start_offset -= left_len;
                        cur_node = &*inner.right;
                    } else {
                        cur_node = &*inner.left;
                        stack.push(inner);
                    }
                }
                NodeTyp::Leaf(leaf) => {
                    break Some(leaf);
                }
            }
        };
        Chunks {
            stack,
            next_leaf,
            start_offset,
            remaining: rope_slice.len(),
        }
    }
}

impl<'a> Iterator for Chunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let next_leaf = self.next_leaf.take()?;
        let ret = if self.start_offset + self.remaining < next_leaf.len() {
            &next_leaf.data[self.start_offset..self.start_offset + self.remaining]
        } else {
            &next_leaf.data[self.start_offset..]
        };
        self.remaining -= ret.len();
        self.start_offset = 0;
        if self.remaining > 0 {
            self.next_leaf = self.stack.pop().map(|inner| {
                let mut cur_node = &*inner.right;
                loop {
                    match &cur_node.typ {
                        NodeTyp::Inner(inner) => {
                            self.stack.push(inner);
                            cur_node = &*inner.left;
                        }
                        NodeTyp::Leaf(leaf) => {
                            break leaf;
                        }
                    }
                }
            });
        }
        Some(ret)
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

    fn insert(&mut self, index: usize, data: &str) {
        assert!(index <= self.len(), "index out of bounds");
        take_mut::take_mut(&mut self.typ, |typ| match typ {
            NodeTyp::Leaf(mut leaf) => {
                if leaf.len() + data.len() <= MAX_NODE_SIZE {
                    leaf.data.insert_str(index, data);
                    NodeTyp::Leaf(leaf)
                } else {
                    leaf.data.insert_str(index, data);
                    NodeTyp::Inner(InnerNode::new_from_str(&leaf.data))
                }
            }
            NodeTyp::Inner(mut inner) => {
                if index < inner.left.len()
                    || (index == inner.left.len() && inner.left.len() < inner.right.len())
                {
                    inner.left.insert(index, data);
                    inner.update_len();
                } else {
                    inner.right.insert(index - inner.left.len(), data);
                    inner.update_len();
                }
                NodeTyp::Inner(inner)
            }
        });
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

    fn new_from_str(s: &str) -> InnerNode {
        let midpoint = s.len() / 2;
        let utf8_mid = utf8::last_utf8_boundary(&s.as_bytes()[..midpoint]);
        let left = if utf8_mid <= MAX_NODE_SIZE {
            Node::new_leaf(LeafNode::new(s[..utf8_mid].to_owned()))
        } else {
            Node::new_inner(InnerNode::new_from_str(&s[..utf8_mid]))
        };
        let right = if s.len() - utf8_mid <= MAX_NODE_SIZE {
            Node::new_leaf(LeafNode::new(s[utf8_mid..].to_owned()))
        } else {
            Node::new_inner(InnerNode::new_from_str(&s[utf8_mid..]))
        };
        InnerNode::new(left, right)
    }

    fn len(&self) -> usize {
        self.length
    }

    fn update_len(&mut self) {
        self.length = self.left.len() + self.right.len();
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

    #[test]
    fn compare_string() {
        let mut buf = String::new();
        let mut do_it = |path| {
            open_file(path).read_to_string(&mut buf).unwrap();
            assert_eq!(Rope::from_reader(open_file(path)).unwrap().to_string(), buf);
            buf.clear();
        };
        do_it("/res/test1.txt");
        do_it("/res/test2.txt");
        do_it("/res/test3.txt");
    }

    #[test]
    fn compare_slice_string() {
        let mut buf = String::new();
        let rope = Rope::from_reader(open_file("/res/test3.txt")).unwrap();
        open_file("/res/test3.txt")
            .read_to_string(&mut buf)
            .unwrap();
        let slice = rope.slice(1000..8002);
        let buf_slice = &buf[1000..8002];
        assert_eq!(&slice.to_string(), buf_slice);
    }

    #[test]
    fn insertion() {
        let mut buf = String::new();
        let mut rope = Rope::from_reader(open_file("/res/test2.txt")).unwrap();
        open_file("/res/test2.txt")
            .read_to_string(&mut buf)
            .unwrap();
        assert_eq!(rope.to_string(), buf);
        rope.insert(10, "====XYZA====");
        buf.insert_str(10, "====XYZA====");
        assert_eq!(rope.to_string(), buf);
        rope.insert_char(200, 'x');
        buf.insert(200, 'x');
        assert_eq!(rope.to_string(), buf);
    }
}
