use super::node::{InnerNode, LeafNode, NodeTyp};
use super::{Rope, RopeSlice};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunks<'a> {
    stack: Vec<&'a InnerNode>,
    next_leaf: Option<&'a LeafNode>,
    start_offset: usize,
    remaining: usize,
}

impl<'a> Chunks<'a> {
    pub(crate) fn new(rope_slice: &RopeSlice<'a>) -> Chunks<'a> {
        if rope_slice.len_bytes() == 0 {
            return Chunks {
                stack: vec![],
                next_leaf: None,
                start_offset: 0,
                remaining: 0,
            };
        }
        let mut stack = Vec::new();
        let mut start_offset = rope_slice.start_offset;
        let mut cur_node = &*rope_slice.rope.root();
        let next_leaf = loop {
            match &cur_node.typ {
                NodeTyp::Inner(inner) => {
                    let left_len = inner.left().len_bytes();
                    if start_offset > left_len {
                        start_offset -= left_len;
                        cur_node = inner.right();
                    } else {
                        cur_node = inner.left();
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
            remaining: rope_slice.len_bytes(),
        }
    }
}

impl<'a> Iterator for Chunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let next_leaf = self.next_leaf.take()?;
        let ret = if self.start_offset + self.remaining < next_leaf.len_bytes() {
            &next_leaf.data()[self.start_offset..self.start_offset + self.remaining]
        } else {
            &next_leaf.data()[self.start_offset..]
        };
        self.remaining -= ret.len();
        self.start_offset = 0;
        if self.remaining > 0 {
            self.next_leaf = self.stack.pop().map(|inner| {
                let mut cur_node = inner.right();
                loop {
                    match &cur_node.typ {
                        NodeTyp::Inner(inner) => {
                            self.stack.push(inner);
                            cur_node = inner.left();
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

pub struct CharIndices<'a> {
    chunks: Chunks<'a>,
    char_indices: Option<std::str::CharIndices<'a>>,
    base: usize,
    next: usize,
}

impl<'a> CharIndices<'a> {
    pub(crate) fn new(slice: &RopeSlice<'a>) -> CharIndices<'a> {
        let mut chunks = slice.chunks();
        let mut next = 0;
        let char_indices = chunks.next().and_then(|s| {
            if s.len() > 0 {
                next += s.len();
                Some(s.char_indices())
            } else {
                None
            }
        });
        CharIndices {
            chunks,
            char_indices,
            base: 0,
            next,
        }
    }
}

impl<'a> Iterator for CharIndices<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<(usize, char)> {
        match &mut self.char_indices {
            None => None,
            Some(char_indices) => {
                if let Some((i, c)) = char_indices.next() {
                    return Some((i + self.base, c));
                }
                self.base = self.next;
                if let Some(chunk) = self.chunks.next() {
                    assert!(!chunk.is_empty());
                    self.next += chunk.len();
                    let mut char_indices = chunk.char_indices();
                    let ret = char_indices.next().map(|(i, c)| (i + self.base, c));
                    self.char_indices = Some(char_indices);
                    ret
                } else {
                    self.char_indices = None;
                    None
                }
            }
        }
    }
}

pub struct Lines<'a> {
    rope: &'a Rope,
    char_indices: Option<CharIndices<'a>>,
    start_offset: usize,
    end_offset: usize,
}

impl<'a> Lines<'a> {
    pub(crate) fn new(slice: &RopeSlice<'a>) -> Lines<'a> {
        Lines {
            rope: slice.rope,
            char_indices: if slice.len_bytes() == 0 {
                None
            } else {
                Some(slice.char_indices())
            },
            start_offset: slice.start_offset,
            end_offset: slice.end_offset,
        }
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = RopeSlice<'a>;

    fn next(&mut self) -> Option<RopeSlice<'a>> {
        match &mut self.char_indices {
            Some(char_indices) => {
                while let Some((i, c)) = char_indices.next() {
                    if c == '\n' {
                        let ret = self.rope.slice(self.start_offset..i + 1);
                        self.start_offset = i + 1;
                        return Some(ret);
                    }
                }
                self.char_indices = None;
                Some(self.rope.slice(self.start_offset..self.end_offset))
            }
            None => None,
        }
    }
}
