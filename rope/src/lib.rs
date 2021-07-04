use std::io::{Read, Result as IOResult};
use std::ops::{Bound, Range, RangeBounds};

mod builder;
mod cow_box;
mod iter;

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

    pub fn remove<R: RangeBounds<usize>>(&mut self, range: R) {
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
        assert!(start <= end, "start cannot be after end");
        assert!(end <= self.len_bytes(), "index out of bounds");
        if start == 0 && end == self.len_bytes() {
            self.root = CowBox::new(Node::new_leaf(LeafNode::new(String::new())));
        } else {
            self.root.remove(start..end);
        }
    }

    pub fn len_bytes(&self) -> usize {
        self.root.len_bytes()
    }

    pub fn len_chars(&self) -> usize {
        self.root.num_chars
    }

    pub fn len_lines(&self) -> usize {
        self.root.num_newlines + 1
    }

    pub fn slice<'a, R: RangeBounds<usize>>(&'a self, range: R) -> RopeSlice<'a> {
        self.whole_slice().slice(range)
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

    fn whole_slice<'a>(&'a self) -> RopeSlice<'a> {
        RopeSlice {
            rope: self,
            start_offset: 0,
            end_offset: self.len_bytes(),
            newlines_before: 0,
            num_newlines: self.root.num_newlines,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RopeSlice<'a> {
    rope: &'a Rope,
    start_offset: usize,
    end_offset: usize,
    newlines_before: usize,
    num_newlines: usize,
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
        let newlines_before = self.rope.root.num_newlines_upto(start_offset);
        let num_newlines = self.rope.root.num_newlines_upto(end_offset) - newlines_before;
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
        self.rope.root.num_chars_upto(self.end_offset)
            - self.rope.root.num_chars_upto(self.start_offset)
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
                .root
                .offset_for_newline(self.newlines_before + index - 1)
                + 1
        };
        let (num_newlines, end_offset) = if index == self.len_lines() - 1 {
            (0, self.end_offset)
        } else {
            (
                1,
                self.rope
                    .root
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
                .root
                .offset_for_newline(self.newlines_before + linum - 1)
                + 1
                - self.start_offset
        }
    }

    pub fn byte_to_line(&self, index: usize) -> usize {
        self.rope.root.num_newlines_upto(self.start_offset + index)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Node {
    typ: NodeTyp,
    num_newlines: usize,
    num_chars: usize,
}

impl Node {
    fn new_inner(inner_node: InnerNode) -> Node {
        let mut ret = Node {
            typ: NodeTyp::Inner(inner_node),
            num_newlines: 0,
            num_chars: 0,
        };
        ret.update_metadata();
        ret
    }

    fn new_leaf(leaf_node: LeafNode) -> Node {
        let mut ret = Node {
            typ: NodeTyp::Leaf(leaf_node),
            num_newlines: 0,
            num_chars: 0,
        };
        ret.update_metadata();
        ret
    }

    fn insert(&mut self, index: usize, data: &str) {
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

    fn remove(&mut self, range: Range<usize>) {
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

    fn len_bytes(&self) -> usize {
        match &self.typ {
            NodeTyp::Inner(inner) => inner.len_bytes(),
            NodeTyp::Leaf(leaf) => leaf.len_bytes(),
        }
    }

    fn num_newlines_upto(&self, index: usize) -> usize {
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

    fn num_chars_upto(&self, index: usize) -> usize {
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

    fn offset_for_newline(&self, newline_idx: usize) -> usize {
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

    fn len_bytes(&self) -> usize {
        self.length
    }

    fn update_len_bytes(&mut self) {
        self.length = self.left.len_bytes() + self.right.len_bytes();
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

    fn count_newlines(&self) -> usize {
        self.data.bytes().filter(|b| *b == b'\n').count()
    }

    fn count_chars(&self) -> usize {
        self.data.chars().count()
    }

    fn len_bytes(&self) -> usize {
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

    struct StrLines<'a> {
        lines: std::iter::Peekable<std::str::Lines<'a>>,
        has_last_line: bool,
    }

    impl<'a> StrLines<'a> {
        fn new(s: &'a str) -> StrLines<'a> {
            let has_last_line = s.ends_with('\n');
            StrLines {
                lines: s.lines().peekable(),
                has_last_line,
            }
        }
    }

    impl<'a> Iterator for StrLines<'a> {
        type Item = String;

        fn next(&mut self) -> Option<String> {
            if let Some(line) = self.lines.next() {
                if !self.has_last_line && self.lines.peek().is_none() {
                    return Some(line.to_owned());
                }
                return Some(line.to_owned() + "\n");
            }
            if self.has_last_line {
                self.has_last_line = false;
                return Some("".to_owned());
            }
            None
        }
    }

    fn line_to_byte(s: &str, linum: usize) -> usize {
        let mut cur_line = 0;
        let bytes = s.as_bytes();
        for i in 0..bytes.len() {
            if cur_line == linum {
                return i;
            }
            if bytes[i] == b'\n' {
                cur_line += 1;
                if cur_line == linum {
                    return i + 1;
                }
            }
        }
        panic!("line index out of bounds");
    }

    fn byte_to_line(s: &str, bidx: usize) -> usize {
        s.bytes().take(bidx).filter(|b| *b == b'\n').count()
    }

    #[test]
    fn len_bytes() {
        let mut buf = String::new();
        let mut do_it = |path, range: Range<usize>| {
            let rope = Rope::from_reader(open_file(path)).unwrap();
            open_file(path).read_to_string(&mut buf).unwrap();
            assert_eq!(rope.len_bytes(), buf.len());
            let ropeslice = rope.slice(range.clone());
            let bufslice = &buf[range];
            assert_eq!(ropeslice.len_bytes(), bufslice.len());
            buf.clear();
        };
        do_it("/res/test1.txt", 5..2408);
        do_it("/res/test2.txt", 2000..3094);
        do_it("/res/test3.txt", 5..8014);
    }

    #[test]
    fn len_chars() {
        let mut buf = String::new();
        let mut do_it = |path, range: Range<usize>| {
            let rope = Rope::from_reader(open_file(path)).unwrap();
            open_file(path).read_to_string(&mut buf).unwrap();
            assert_eq!(rope.len_chars(), buf.chars().count());
            let ropeslice = rope.slice(range.clone());
            let bufslice = &buf[range];
            assert_eq!(ropeslice.len_chars(), bufslice.chars().count());
            buf.clear();
        };
        do_it("/res/test1.txt", 5..2408);
        do_it("/res/test2.txt", 2000..3094);
        do_it("/res/test3.txt", 5..8014);
    }

    #[test]
    #[should_panic(expected = "slice index out of bounds")]
    fn slice_fail() {
        let rope = Rope::from_reader(open_file("/res/test1.txt")).unwrap();
        assert_eq!(rope.len_bytes(), 2412);
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
    fn compare_iterators_empty() {
        let rope = Rope::new();
        assert!(rope.chars().eq("".chars()));
        assert!(rope.char_indices().eq("".char_indices()));
        assert!(rope
            .lines()
            .map(|line| line.to_string())
            .eq(StrLines::new("")));
    }

    #[test]
    fn compare_iterators() {
        let mut buf = String::new();
        let mut do_it = |path| {
            open_file(path).read_to_string(&mut buf).unwrap();
            let rope = Rope::from_reader(open_file(path)).unwrap();
            assert!(rope.chars().eq(buf.chars()));
            assert!(rope.char_indices().eq(buf.char_indices()));
            assert!(rope
                .lines()
                .map(|line| line.to_string())
                .eq(StrLines::new(&buf)));
            buf.clear();
        };
        do_it("/res/test1.txt");
        do_it("/res/test2.txt");
        do_it("/res/test3.txt");
    }

    #[test]
    fn insertion_empty() {
        let mut rope = Rope::new();
        assert_eq!(rope.to_string(), "".to_owned());
        rope.insert(0, "====XYZA====");
        assert_eq!(rope.to_string(), "====XYZA====");
        rope.insert_char(4, 'x');
        assert_eq!(rope.to_string(), "====xXYZA====");
    }

    #[test]
    fn insertion() {
        let mut buf = String::new();
        let mut rope = Rope::from_reader(open_file("/res/test3.txt")).unwrap();
        open_file("/res/test3.txt")
            .read_to_string(&mut buf)
            .unwrap();
        assert_eq!(rope.to_string(), buf);
        rope.insert(1000, "====XYZA====");
        buf.insert_str(1000, "====XYZA====");
        assert_eq!(rope.to_string(), buf);
        rope.insert_char(8014, 'x');
        buf.insert(8014, 'x');
        assert_eq!(rope.to_string(), buf);
    }

    #[test]
    fn remove() {
        let mut buf = String::new();
        let mut do_it = |path, range: Range<usize>| {
            open_file(path).read_to_string(&mut buf).unwrap();
            let mut rope = Rope::from_reader(open_file(path)).unwrap();
            buf.replace_range(range.clone(), "");
            rope.remove(range);
            assert!(rope.chars().eq(buf.chars()));
            assert!(rope.char_indices().eq(buf.char_indices()));
            assert!(rope
                .lines()
                .map(|line| line.to_string())
                .eq(StrLines::new(&buf)));
            buf.clear();
        };
        do_it("/res/test1.txt", 10..20);
        do_it("/res/test2.txt", 0..4096);
        do_it("/res/test3.txt", 1000..8002);
    }

    #[test]
    fn len_lines() {
        let mut buf = String::new();
        let mut do_it = |path, range: Range<usize>| {
            open_file(path).read_to_string(&mut buf).unwrap();
            let mut rope = Rope::from_reader(open_file(path)).unwrap();
            let diff = if buf.ends_with('\n') { 1 } else { 0 };
            assert_eq!(rope.len_lines() - diff, buf.lines().count());
            buf.replace_range(range.clone(), "");
            rope.remove(range);
            let diff = if buf.ends_with('\n') { 1 } else { 0 };
            assert_eq!(rope.len_lines() - diff, buf.lines().count());
            buf.clear();
        };
        do_it("/res/test1.txt", 10..20);
        do_it("/res/test2.txt", 0..4096);
        do_it("/res/test3.txt", 1000..8002);
    }

    #[test]
    fn slice_len_lines() {
        let mut buf = String::new();
        let mut do_it = |path, del_range: Range<usize>, slice_range: Range<usize>| {
            open_file(path).read_to_string(&mut buf).unwrap();
            let mut rope = Rope::from_reader(open_file(path)).unwrap();
            let bufslice = &buf[slice_range.clone()];
            let ropeslice = rope.slice(slice_range.clone());
            let diff = if bufslice.ends_with('\n') { 1 } else { 0 };
            assert_eq!(ropeslice.len_lines() - diff, bufslice.lines().count());
            buf.replace_range(del_range.clone(), "");
            rope.remove(del_range);
            let bufslice = &buf[slice_range.clone()];
            let ropeslice = rope.slice(slice_range.clone());
            let diff = if bufslice.ends_with('\n') { 1 } else { 0 };
            assert_eq!(ropeslice.len_lines() - diff, bufslice.lines().count());
            buf.clear();
        };
        do_it("/res/test1.txt", 10..20, 5..200);
        do_it("/res/test2.txt", 0..4096, 0..5);
        do_it("/res/test3.txt", 1000..8002, 5..2006);
    }

    #[test]
    fn line_indices() {
        let mut buf = String::new();
        let mut do_it = |path, range: Range<usize>| {
            open_file(path).read_to_string(&mut buf).unwrap();
            let mut rope = Rope::from_reader(open_file(path)).unwrap();
            assert!((0..rope.len_lines())
                .map(|i| rope.line(i).to_string())
                .eq(StrLines::new(&buf)));
            buf.replace_range(range.clone(), "");
            rope.remove(range);
            assert!((0..rope.len_lines())
                .map(|i| rope.line(i).to_string())
                .eq(StrLines::new(&buf)));
            buf.clear();
        };
        do_it("/res/test1.txt", 10..20);
        do_it("/res/test2.txt", 0..4096);
        do_it("/res/test3.txt", 1000..8002);
    }

    #[test]
    fn slice_line_indices() {
        let mut buf = String::new();
        let mut do_it = |path, del_range: Range<usize>, slice_range: Range<usize>| {
            open_file(path).read_to_string(&mut buf).unwrap();
            let mut rope = Rope::from_reader(open_file(path)).unwrap();
            let bufslice = &buf[slice_range.clone()];
            let ropeslice = rope.slice(slice_range.clone());
            assert!((0..ropeslice.len_lines())
                .map(|i| ropeslice.line(i).to_string())
                .eq(StrLines::new(bufslice)));
            buf.replace_range(del_range.clone(), "");
            rope.remove(del_range);
            let bufslice = &buf[slice_range.clone()];
            let ropeslice = rope.slice(slice_range.clone());
            assert!((0..ropeslice.len_lines())
                .map(|i| ropeslice.line(i).to_string())
                .eq(StrLines::new(bufslice)));
            buf.clear();
        };
        do_it("/res/test1.txt", 10..20, 5..200);
        do_it("/res/test2.txt", 0..4096, 0..5);
        do_it("/res/test3.txt", 1000..8002, 5..2006);
    }

    #[test]
    fn line_byte_indices() {
        let mut buf = String::new();
        let mut do_it =
            |path, range: Range<usize>, byte_indices: &[usize], line_indices: &[usize]| {
                open_file(path).read_to_string(&mut buf).unwrap();
                let mut rope = Rope::from_reader(open_file(path)).unwrap();
                for &li in line_indices.iter() {
                    if li < rope.len_lines() {
                        assert_eq!(rope.line_to_byte(li), line_to_byte(&buf, li));
                    }
                }
                for &bi in byte_indices.iter() {
                    if bi < rope.len_bytes() {
                        assert_eq!(rope.byte_to_line(bi), byte_to_line(&buf, bi));
                    }
                }
                buf.replace_range(range.clone(), "");
                rope.remove(range);
                for &li in line_indices.iter() {
                    if li < rope.len_lines() {
                        assert_eq!(rope.line_to_byte(li), line_to_byte(&buf, li));
                    }
                }
                for &bi in byte_indices.iter() {
                    if bi < rope.len_bytes() {
                        assert_eq!(rope.byte_to_line(bi), byte_to_line(&buf, bi));
                    }
                }
                buf.clear();
            };
        do_it("/res/test1.txt", 10..20, &[20, 100, 1000], &[2, 10]);
        do_it("/res/test2.txt", 0..4096, &[50, 100], &[5, 10]);
        do_it("/res/test3.txt", 1000..8002, &[300, 4000], &[10, 300, 500]);
    }

    #[test]
    fn slice_byte_indices() {
        let mut buf = String::new();
        let mut do_it = |path,
                         del_range: Range<usize>,
                         slice_range: Range<usize>,
                         byte_indices: &[usize],
                         line_indices: &[usize]| {
            open_file(path).read_to_string(&mut buf).unwrap();
            let mut rope = Rope::from_reader(open_file(path)).unwrap();
            let bufslice = &buf[slice_range.clone()];
            let ropeslice = rope.slice(slice_range.clone());
            for &li in line_indices.iter() {
                if li < ropeslice.len_lines() {
                    assert_eq!(ropeslice.line_to_byte(li), line_to_byte(&bufslice, li));
                }
            }
            for &bi in byte_indices.iter() {
                if bi < ropeslice.len_bytes() {
                    assert_eq!(ropeslice.byte_to_line(bi), byte_to_line(&bufslice, bi));
                }
            }
            buf.replace_range(del_range.clone(), "");
            rope.remove(del_range);
            let bufslice = &buf[slice_range.clone()];
            let ropeslice = rope.slice(slice_range.clone());
            for &li in line_indices.iter() {
                if li < ropeslice.len_lines() {
                    assert_eq!(ropeslice.line_to_byte(li), line_to_byte(&bufslice, li));
                }
            }
            for &bi in byte_indices.iter() {
                if bi < ropeslice.len_bytes() {
                    assert_eq!(ropeslice.byte_to_line(bi), byte_to_line(&bufslice, bi));
                }
            }
            buf.clear();
        };
        do_it("/res/test1.txt", 10..20, 5..200, &[20, 100, 1000], &[2, 10]);
        do_it("/res/test2.txt", 0..4096, 0..5, &[50, 100], &[5, 10]);
        do_it(
            "/res/test3.txt",
            1000..8002,
            6..2006,
            &[20, 1000],
            &[20, 100],
        );
    }
}
