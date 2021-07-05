use std::ops::Range;

use super::cow_box::CowBox;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Node {
    pub(crate) typ: NodeTyp,
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

    pub(crate) fn insert(&mut self, char_index: usize, data: &str) {
        match &mut self.typ {
            NodeTyp::Leaf(leaf) => {
                if leaf.num_bytes() + data.len() > MAX_NODE_SIZE {
                    self.typ = NodeTyp::Inner(leaf.insert_and_split(char_index, data));
                } else {
                    leaf.insert(char_index, data);
                }
            }
            NodeTyp::Inner(inner) => inner.insert(char_index, data),
        };
    }

    pub(crate) fn remove(&mut self, char_range: Range<usize>) {
        match &mut self.typ {
            NodeTyp::Leaf(leaf) => leaf.remove(char_range),
            NodeTyp::Inner(inner) => {
                self.typ = inner.remove(char_range);
            }
        };
    }

    pub(crate) fn num_bytes(&self) -> usize {
        match &self.typ {
            NodeTyp::Inner(inner) => inner.num_bytes,
            NodeTyp::Leaf(leaf) => leaf.num_bytes(),
        }
    }

    pub(crate) fn num_chars(&self) -> usize {
        match &self.typ {
            NodeTyp::Inner(inner) => inner.num_chars,
            NodeTyp::Leaf(leaf) => leaf.num_chars,
        }
    }

    pub(crate) fn num_newlines(&self) -> usize {
        match &self.typ {
            NodeTyp::Inner(inner) => inner.num_newlines,
            NodeTyp::Leaf(leaf) => leaf.num_newlines,
        }
    }

    pub(crate) fn num_newlines_upto_bidx(&self, bidx: usize) -> usize {
        match &self.typ {
            NodeTyp::Leaf(leaf) => leaf.num_newlines_upto_bidx(bidx),
            NodeTyp::Inner(inner) => inner.num_newlines_upto(bidx),
        }
    }

    pub(crate) fn num_chars_upto_bidx(&self, bidx: usize) -> usize {
        match &self.typ {
            NodeTyp::Leaf(leaf) => leaf.num_chars_upto_bidx(bidx),
            NodeTyp::Inner(inner) => inner.num_chars_upto_bidx(bidx),
        }
    }

    pub(crate) fn bidx_for_newline(&self, newline_idx: usize) -> usize {
        match &self.typ {
            NodeTyp::Leaf(leaf) => leaf.bidx_for_newline(newline_idx),
            NodeTyp::Inner(inner) => inner.bidx_for_newline(newline_idx),
        }
    }

    pub(crate) fn bidx_for_char(&self, cidx: usize) -> usize {
        match &self.typ {
            NodeTyp::Leaf(leaf) => leaf.bidx_for_char(cidx),
            NodeTyp::Inner(inner) => inner.bidx_for_char(cidx),
        }
    }
}

impl From<LeafNode> for Node {
    fn from(leaf_node: LeafNode) -> Node {
        Node {
            typ: NodeTyp::Leaf(leaf_node),
        }
    }
}

impl From<InnerNode> for Node {
    fn from(inner_node: InnerNode) -> Node {
        Node {
            typ: NodeTyp::Inner(inner_node),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NodeTyp {
    Inner(InnerNode),
    Leaf(LeafNode),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InnerNode {
    num_bytes: usize,
    num_chars: usize,
    num_newlines: usize,
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
        let num_bytes = left.num_bytes() + right.num_bytes();
        let num_chars = left.num_chars() + right.num_chars();
        let num_newlines = left.num_newlines() + right.num_newlines();
        InnerNode {
            num_bytes,
            num_chars,
            num_newlines,
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

    fn insert(&mut self, char_index: usize, data: &str) {
        if char_index < self.left.num_chars()
            || (char_index == self.left.num_chars()
                && self.left.num_chars() < self.right.num_chars())
        {
            self.left.insert(char_index, data)
        } else {
            self.right.insert(char_index - self.left.num_chars(), data)
        }
        self.update_metadata();
    }

    fn remove(&mut self, char_range: Range<usize>) -> NodeTyp {
        if char_range.is_empty() {
            return NodeTyp::Inner(self.clone());
        }
        assert!(char_range.end <= self.num_chars, "char index out of bounds");
        assert!(
            char_range.start > 0 || char_range.end < self.num_chars,
            "full range deletion should be handled earlier"
        );
        let left_num_chars = self.left.num_chars();
        if char_range.end <= left_num_chars {
            if char_range.start == 0 && char_range.end == left_num_chars {
                self.right.typ.clone()
            } else {
                self.left.remove(char_range);
                self.update_metadata();
                NodeTyp::Inner(self.clone())
            }
        } else if char_range.start >= left_num_chars {
            if char_range.start == left_num_chars && char_range.end == self.num_chars {
                self.right.typ.clone()
            } else {
                self.right.remove(char_range);
                self.update_metadata();
                NodeTyp::Inner(self.clone())
            }
        } else if char_range.start == 0 {
            self.right.remove(0..char_range.end - left_num_chars);
            self.right.typ.clone()
        } else if char_range.end == self.num_chars {
            self.left.remove(char_range.start..left_num_chars);
            self.left.typ.clone()
        } else {
            self.left.remove(char_range.start..left_num_chars);
            self.right.remove(0..char_range.end - left_num_chars);
            self.update_metadata();
            NodeTyp::Inner(self.clone())
        }
    }

    fn num_newlines_upto(&self, bidx: usize) -> usize {
        if bidx <= self.left.num_bytes() {
            self.left.num_newlines_upto_bidx(bidx)
        } else {
            self.left.num_newlines()
                + self
                    .right
                    .num_newlines_upto_bidx(bidx - self.left.num_bytes())
        }
    }

    fn num_chars_upto_bidx(&self, bidx: usize) -> usize {
        if bidx <= self.left.num_bytes() {
            self.left.num_chars_upto_bidx(bidx)
        } else {
            self.left.num_chars() + self.right.num_chars_upto_bidx(bidx - self.left.num_bytes())
        }
    }

    fn bidx_for_newline(&self, newline_idx: usize) -> usize {
        if newline_idx < self.left.num_newlines() {
            self.left.bidx_for_newline(newline_idx)
        } else {
            self.left.num_bytes()
                + self
                    .right
                    .bidx_for_newline(newline_idx - self.left.num_newlines())
        }
    }

    fn bidx_for_char(&self, cidx: usize) -> usize {
        if cidx < self.left.num_chars() {
            self.left.bidx_for_char(cidx)
        } else {
            self.left.num_bytes() + self.right.bidx_for_char(cidx - self.left.num_chars())
        }
    }

    fn update_metadata(&mut self) {
        self.num_bytes = self.left.num_bytes() + self.right.num_bytes();
        self.num_chars = self.left.num_chars() + self.right.num_chars();
        self.num_newlines = self.left.num_newlines() + self.right.num_newlines();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LeafNode {
    data: String,
    num_chars: usize,
    num_newlines: usize,
}

impl LeafNode {
    pub(crate) fn data(&self) -> &str {
        &self.data
    }

    pub(crate) fn num_bytes(&self) -> usize {
        self.data.len()
    }

    fn new(data: String) -> LeafNode {
        assert!(data.len() <= MAX_NODE_SIZE);
        let num_chars = Self::count_chars(&data);
        let num_newlines = Self::count_newlines(&data);
        LeafNode {
            data,
            num_chars,
            num_newlines,
        }
    }

    fn insert(&mut self, char_index: usize, data: &str) {
        assert!(
            self.num_bytes() + data.len() <= MAX_NODE_SIZE,
            "too much data being inserted"
        );
        assert!(char_index <= self.num_chars, "char index out of bounds");
        let byte_index = if char_index == self.num_chars {
            self.num_bytes()
        } else {
            self.data.char_indices().nth(char_index).unwrap().0
        };
        self.data.insert_str(byte_index, data);
        self.num_chars += Self::count_chars(data);
        self.num_newlines += Self::count_newlines(data);
    }

    fn insert_and_split(&self, char_index: usize, data: &str) -> InnerNode {
        assert!(char_index <= self.num_chars, "char index out of bounds");
        let mut copy = self.data.clone();
        let byte_index = if char_index == self.num_chars {
            self.num_bytes()
        } else {
            copy.char_indices().nth(char_index).unwrap().0
        };
        copy.insert_str(byte_index, data);
        InnerNode::new_from_str(&copy)
    }

    fn remove(&mut self, char_range: Range<usize>) {
        if char_range.is_empty() {
            return;
        }
        assert!(char_range.end <= self.num_chars, "char index out of bounds");
        assert!(
            char_range.start > 0 || char_range.end < self.num_chars,
            "full range deletion should be handled earlier"
        );
        let start_bidx = self.data.char_indices().nth(char_range.start).unwrap().0;
        let end_bidx = if char_range.end == self.num_chars {
            self.num_bytes()
        } else {
            self.data[start_bidx..]
                .char_indices()
                .nth(char_range.len())
                .unwrap()
                .0
                + start_bidx
        };
        self.num_chars -= char_range.len();
        self.num_newlines -= Self::count_newlines(&self.data[start_bidx..end_bidx]);
        self.data.replace_range(start_bidx..end_bidx, "");
    }

    fn num_newlines_upto_bidx(&self, bidx: usize) -> usize {
        assert!(bidx <= self.num_bytes());
        if bidx == self.num_bytes() {
            self.num_newlines
        } else {
            Self::count_newlines(&self.data[..bidx])
        }
    }

    fn num_chars_upto_bidx(&self, bidx: usize) -> usize {
        assert!(bidx <= self.num_bytes());
        if bidx == self.num_bytes() {
            self.num_chars
        } else {
            Self::count_chars(&self.data[..bidx])
        }
    }

    fn bidx_for_newline(&self, newline_idx: usize) -> usize {
        assert!(newline_idx < self.num_newlines);
        self.data
            .bytes()
            .enumerate()
            .filter_map(|(i, b)| if b == b'\n' { Some(i) } else { None })
            .nth(newline_idx)
            .unwrap()
    }

    fn bidx_for_char(&self, cidx: usize) -> usize {
        assert!(cidx < self.num_chars);
        self.data.char_indices().nth(cidx).unwrap().0
    }

    fn count_chars(data: &str) -> usize {
        data.chars().count()
    }

    fn count_newlines(data: &str) -> usize {
        data.bytes().filter(|b| *b == b'\n').count()
    }
}

pub(crate) const MAX_NODE_SIZE: usize = 4096;

#[cfg(test)]
mod tests {
    use super::*;

    fn char_to_byte(s: &str, char_idx: usize) -> usize {
        s.char_indices()
            .map(|x| x.0)
            .nth(char_idx)
            .unwrap_or(s.len())
    }

    #[test]
    fn leaf_node() {
        let mut data = "abc\ndef ði ı\nntəˈnæʃənəl fəˈnɛtık əsoʊsiˈeıʃn".to_owned();
        let mut leaf = Node::new_leaf(data.clone());
        assert_eq!(leaf.num_bytes(), data.len());
        assert_eq!(leaf.num_chars(), data.chars().count());
        assert_eq!(
            leaf.num_newlines(),
            data.chars().filter(|c| *c == '\n').count()
        );
        let newstr = "XY\nZΣὲ γνωρ\nA";
        leaf.insert(10, newstr);
        data.insert_str(char_to_byte(&data, 10), newstr);
        assert_eq!(leaf.num_bytes(), data.len());
        assert_eq!(leaf.num_chars(), data.chars().count());
        assert_eq!(
            leaf.num_newlines(),
            data.chars().filter(|c| *c == '\n').count()
        );
        leaf.remove(20..30);
        data.replace_range(char_to_byte(&data, 20)..char_to_byte(&data, 30), "");
        assert_eq!(leaf.num_bytes(), data.len());
        assert_eq!(leaf.num_chars(), data.chars().count());
        assert_eq!(
            leaf.num_newlines(),
            data.chars().filter(|c| *c == '\n').count()
        );
    }

    #[test]
    fn inner_node() {
        let mut data = "abc\ndef ði ı\nntəˈnæʃənəl fəˈnɛtık əsoʊsiˈeıʃn".repeat(1000);
        let mut inner = Node::from(InnerNode::new_from_str(&data));
        assert_eq!(inner.num_bytes(), data.len());
        assert_eq!(inner.num_chars(), data.chars().count());
        assert_eq!(
            inner.num_newlines(),
            data.chars().filter(|c| *c == '\n').count()
        );
        let newstr = "XY\nZΣὲ γνωρ\nA".repeat(1000);
        inner.insert(1000, &newstr);
        data.insert_str(char_to_byte(&data, 1000), &newstr);
        assert_eq!(inner.num_bytes(), data.len());
        assert_eq!(inner.num_chars(), data.chars().count());
        assert_eq!(
            inner.num_newlines(),
            data.chars().filter(|c| *c == '\n').count()
        );
        inner.remove(2000..8000);
        data.replace_range(char_to_byte(&data, 2000)..char_to_byte(&data, 8000), "");
        assert_eq!(inner.num_bytes(), data.len());
        assert_eq!(inner.num_chars(), data.chars().count());
        assert_eq!(
            inner.num_newlines(),
            data.chars().filter(|c| *c == '\n').count()
        );
    }
}
