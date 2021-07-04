use std::ops::Range;

use super::cow_box::CowBox;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Node {
    pub(crate) typ: NodeTyp,
    num_newlines: usize,
    num_chars: usize,
}

impl Node {
    pub(crate) fn new_leaf(data: String) -> Node {
        let leaf_node = LeafNode::new(data);
        Node::from(leaf_node)
    }

    pub(crate) fn new_inner(left: Node, right: Node) -> Node {
        let inner_node = InnerNode::new(left, right);
        Node::from(inner_node)
    }

    pub(crate) fn insert(&mut self, index: usize, data: &str) {
        assert!(index <= self.len_bytes(), "index out of bounds");
        match &mut self.typ {
            NodeTyp::Leaf(leaf) => {
                if utf8::last_utf8_boundary(&leaf.data.as_bytes()[..index]) != index {
                    panic!("indexing in the middle of a UTF-8 character");
                }
                if leaf.len_bytes() + data.len() <= MAX_NODE_SIZE {
                    leaf.data.insert_str(index, data);
                } else {
                    leaf.data.insert_str(index, data);
                    self.typ = NodeTyp::Inner(InnerNode::new_from_str(&leaf.data));
                }
            }
            NodeTyp::Inner(inner) => {
                if index < inner.left.len_bytes()
                    || (index == inner.left.len_bytes()
                        && inner.left.len_bytes() < inner.right.len_bytes())
                {
                    inner.left.insert(index, data);
                } else {
                    inner.right.insert(index - inner.left.len_bytes(), data);
                }
            }
        };
        self.update_metadata();
    }

    pub(crate) fn remove(&mut self, range: Range<usize>) {
        assert!(range.start <= range.end, "start cannot be after end");
        assert!(range.end <= self.len_bytes(), "index out of bounds");
        assert!(
            range.start > 0 || range.end < self.len_bytes(),
            "full range deletion should be handled earlier"
        );
        match &mut self.typ {
            NodeTyp::Leaf(leaf) => {
                leaf.data.replace_range(range.clone(), "");
            }
            NodeTyp::Inner(inner) => {
                let left_len = inner.left.len_bytes();
                if range.end <= left_len {
                    if range.start == 0 && range.end == left_len {
                        self.typ = inner.right.typ.clone();
                    } else {
                        inner.left.remove(range.clone());
                    }
                } else if range.start >= left_len {
                    if range.start == left_len && range.end == inner.len_bytes() {
                        self.typ = inner.left.typ.clone();
                    } else {
                        inner
                            .right
                            .remove(range.start - left_len..range.end - left_len);
                    }
                } else {
                    inner.left.remove(range.start..left_len);
                    inner.right.remove(0..range.end - left_len);
                }
            }
        };
        self.update_metadata();
    }

    pub(crate) fn len_bytes(&self) -> usize {
        match &self.typ {
            NodeTyp::Inner(inner) => inner.len_bytes(),
            NodeTyp::Leaf(leaf) => leaf.len_bytes(),
        }
    }

    pub(crate) fn num_chars(&self) -> usize {
        self.num_chars
    }

    pub(crate) fn num_newlines(&self) -> usize {
        self.num_newlines
    }

    pub(crate) fn num_newlines_upto(&self, index: usize) -> usize {
        assert!(
            index <= self.len_bytes(),
            "index ({}) <= self.len_bytes() ({})",
            index,
            self.len_bytes()
        );
        if index == self.len_bytes() {
            return self.num_newlines;
        }
        match &self.typ {
            NodeTyp::Leaf(leaf) => leaf.data[..index].bytes().filter(|b| *b == b'\n').count(),
            NodeTyp::Inner(inner) => {
                if index <= inner.left.len_bytes() {
                    inner.left.num_newlines_upto(index)
                } else {
                    inner.left.num_newlines
                        + inner
                            .right
                            .num_newlines_upto(index - inner.left.len_bytes())
                }
            }
        }
    }

    pub(crate) fn num_chars_upto(&self, index: usize) -> usize {
        assert!(
            index <= self.len_bytes(),
            "index ({}) <= self.len_bytes() ({})",
            index,
            self.len_bytes()
        );
        if index == self.len_bytes() {
            return self.num_chars;
        }
        match &self.typ {
            NodeTyp::Leaf(leaf) => leaf.data[..index].chars().count(),
            NodeTyp::Inner(inner) => {
                if index <= inner.left.len_bytes() {
                    inner.left.num_chars_upto(index)
                } else {
                    inner.left.num_chars
                        + inner.right.num_chars_upto(index - inner.left.len_bytes())
                }
            }
        }
    }

    pub(crate) fn offset_for_newline(&self, newline_idx: usize) -> usize {
        assert!(
            newline_idx < self.num_newlines,
            "newline_idx ({}) < self.num_newlines ({})",
            newline_idx,
            self.num_newlines
        );
        match &self.typ {
            NodeTyp::Leaf(leaf) => leaf
                .data
                .bytes()
                .enumerate()
                .filter_map(|(i, b)| if b == b'\n' { Some(i) } else { None })
                .nth(newline_idx)
                .unwrap(),
            NodeTyp::Inner(inner) => {
                if newline_idx < inner.left.num_newlines {
                    inner.left.offset_for_newline(newline_idx)
                } else {
                    inner.left.len_bytes()
                        + inner
                            .right
                            .offset_for_newline(newline_idx - inner.left.num_newlines)
                }
            }
        }
    }

    fn update_metadata(&mut self) {
        match &mut self.typ {
            NodeTyp::Inner(inner) => {
                inner.update_len_bytes();
                self.num_newlines = inner.left.num_newlines + inner.right.num_newlines;
                self.num_chars = inner.left.num_chars + inner.right.num_chars;
            }
            NodeTyp::Leaf(leaf) => {
                self.num_newlines = leaf.count_newlines();
                self.num_chars = leaf.count_chars();
            }
        }
    }
}

impl From<LeafNode> for Node {
    fn from(leaf_node: LeafNode) -> Node {
        let mut ret = Node {
            typ: NodeTyp::Leaf(leaf_node),
            num_newlines: 0,
            num_chars: 0,
        };
        ret.update_metadata();
        ret
    }
}

impl From<InnerNode> for Node {
    fn from(inner_node: InnerNode) -> Node {
        let mut ret = Node {
            typ: NodeTyp::Inner(inner_node),
            num_newlines: 0,
            num_chars: 0,
        };
        ret.update_metadata();
        ret
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NodeTyp {
    Inner(InnerNode),
    Leaf(LeafNode),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InnerNode {
    length: usize,
    left: CowBox<Node>,
    right: CowBox<Node>,
}

impl InnerNode {
    pub(crate) fn left(&self) -> &Node {
        &*self.left
    }

    pub(crate) fn right(&self) -> &Node {
        &*self.right
    }

    fn new(left: Node, right: Node) -> InnerNode {
        let length = left.len_bytes() + right.len_bytes();
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
            Node::new_leaf(s[..utf8_mid].to_owned())
        } else {
            Node::from(InnerNode::new_from_str(&s[..utf8_mid]))
        };
        let right = if s.len() - utf8_mid <= MAX_NODE_SIZE {
            Node::new_leaf(s[utf8_mid..].to_owned())
        } else {
            Node::from(InnerNode::new_from_str(&s[utf8_mid..]))
        };
        InnerNode::new(left, right)
    }

    fn len_bytes(&self) -> usize {
        self.length
    }

    fn update_len_bytes(&mut self) {
        self.length = self.left.len_bytes() + self.right.len_bytes();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LeafNode {
    data: String,
}

impl LeafNode {
    pub(crate) fn data(&self) -> &str {
        &self.data
    }

    pub(crate) fn len_bytes(&self) -> usize {
        self.data.len()
    }

    fn new(data: String) -> LeafNode {
        LeafNode { data }
    }

    fn count_newlines(&self) -> usize {
        self.data.bytes().filter(|b| *b == b'\n').count()
    }

    fn count_chars(&self) -> usize {
        self.data.chars().count()
    }
}

pub(crate) const MAX_NODE_SIZE: usize = 4096;
