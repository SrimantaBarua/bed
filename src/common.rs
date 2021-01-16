// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::env;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::Chars as StrChars;

use ropey::{iter::Chars as RopeChars, iter::Chunks, str_utils::byte_to_char_idx, RopeSlice};
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::{
    GraphemeCursor, GraphemeIncomplete, GraphemeIndices, UnicodeSegmentation,
};

// -------- Dummy data types --------

pub(crate) struct PixelSize;
pub(crate) struct TextureSize;

// -------- Absolute paths --------

#[derive(Clone, Eq, PartialEq, Hash)]
pub(crate) struct AbsPath(PathBuf);

impl AsRef<Path> for AbsPath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl AbsPath {
    pub(crate) fn from<P: AsRef<Path>>(p: P) -> AbsPath {
        let path = p.as_ref();
        if path.is_absolute() {
            AbsPath(path.to_path_buf())
        } else if path.starts_with("~") {
            let path = path.strip_prefix("~").unwrap();
            let mut home = PathBuf::from(env::var("HOME").expect("failed to get HOME directory"));
            home.push(path);
            AbsPath(home)
        } else {
            let mut cur_dir = env::current_dir().expect("failed to get current directory");
            cur_dir.push(path);
            AbsPath(cur_dir)
        }
    }

    pub(crate) fn file_name(&self) -> Option<&str> {
        self.0.file_name().and_then(|s| s.to_str())
    }
}

// -------- Rope stuff --------

pub(crate) fn rope_trim_newlines<'a>(line: RopeSlice<'a>) -> RopeSlice<'a> {
    let mut nchars = line.len_chars();
    let mut chars = line.chars_at(line.len_chars());
    while let Some(c) = chars.prev() {
        match c {
            '\n' | '\x0b' | '\x0c' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}' => nchars -= 1,
            _ => break,
        }
    }
    line.slice(..nchars)
}

pub(crate) trait SliceRange: Clone + Default {
    fn push(&mut self, c: char);
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn from_raw(range: Range<usize>) -> Self;
    fn shift(&mut self, c: char) {
        self.push(c);
        self.clear();
    }
}

pub(crate) trait RopeOrStr: std::fmt::Display {
    type CharIter: Iterator<Item = char>;
    type GraphemeIter: Iterator<Item = usize>;
    type SliceRange: SliceRange;

    fn char_iter(&self) -> Self::CharIter;
    fn blen(&self) -> usize;
    fn string(&self) -> String;
    fn grapheme_idxs(&self) -> Self::GraphemeIter;
    fn slice_with(&self, range: Self::SliceRange) -> Self;
}

impl<'a> RopeOrStr for RopeSlice<'a> {
    type CharIter = RopeChars<'a>;
    type GraphemeIter = RopeGraphemeIndices<'a>;
    type SliceRange = RopeSliceRange;

    fn char_iter(&self) -> RopeChars<'a> {
        self.chars()
    }

    fn blen(&self) -> usize {
        self.len_bytes()
    }

    fn string(&self) -> String {
        self.to_string()
    }

    fn grapheme_idxs(&self) -> RopeGraphemeIndices<'a> {
        RopeGraphemeIndices(RopeGraphemes::new(self))
    }

    fn slice_with(&self, range: RopeSliceRange) -> RopeSlice<'a> {
        self.slice(range.0)
    }
}

#[derive(Clone, Default)]
pub(crate) struct RopeSliceRange(Range<usize>);

impl SliceRange for RopeSliceRange {
    fn push(&mut self, _: char) {
        self.0.end += 1;
    }

    fn clear(&mut self) {
        self.0.start = self.0.end
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.end - self.0.start
    }

    fn from_raw(range: Range<usize>) -> Self {
        RopeSliceRange(range)
    }
}

pub(crate) struct RopeGraphemeIndices<'a>(RopeGraphemes<'a>);

impl<'a> Iterator for RopeGraphemeIndices<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.0.next().map(|(j, _)| j)
    }
}

impl<'a> RopeOrStr for &'a str {
    type CharIter = StrChars<'a>;
    type GraphemeIter = StringGraphemeIndices<'a>;
    type SliceRange = StrSliceRange;

    fn char_iter(&self) -> StrChars<'a> {
        self.chars()
    }

    fn blen(&self) -> usize {
        self.len()
    }

    fn string(&self) -> String {
        self.to_string()
    }

    fn grapheme_idxs(&self) -> StringGraphemeIndices<'a> {
        StringGraphemeIndices(self.grapheme_indices(true))
    }

    fn slice_with(&self, range: StrSliceRange) -> &'a str {
        &self[range.0]
    }
}

#[derive(Clone, Default)]
pub(crate) struct StrSliceRange(Range<usize>);

impl SliceRange for StrSliceRange {
    fn push(&mut self, c: char) {
        self.0.end += c.len_utf8();
    }

    fn clear(&mut self) {
        self.0.start = self.0.end
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.end - self.0.start
    }

    fn from_raw(range: Range<usize>) -> Self {
        StrSliceRange(range)
    }
}

pub(crate) struct StringGraphemeIndices<'a>(GraphemeIndices<'a>);

impl<'a> Iterator for StringGraphemeIndices<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.0.next().map(|(j, _)| j)
    }
}

#[derive(Eq, PartialEq)]
pub(crate) enum SplitCbRes {
    Continue,
    Stop,
}

// Split text into runs and spaces
pub(crate) fn split_text<S, R, SR>(line: &SR, tab_width: usize, mut space_cb: S, mut run_cb: R)
where
    S: FnMut(usize) -> SplitCbRes,
    R: FnMut(&SR) -> SplitCbRes,
    SR: RopeOrStr,
{
    let mut last_script = None;
    let mut x = 0;
    let mut range = SR::SliceRange::default();
    let mut num_spaces = 0;
    for c in line.char_iter() {
        match c {
            '\n' | '\r' | '\x0b' | '\x0c' | '\u{85}' | '\u{2028}' | '\u{2029}' => break,
            ' ' => {
                if !range.is_empty() {
                    if run_cb(&line.slice_with(range.clone())) == SplitCbRes::Stop {
                        return;
                    }
                    range.clear();
                    last_script = None;
                }
                range.shift(' ');
                x += 1;
                num_spaces += 1;
            }
            '\t' => {
                if !range.is_empty() {
                    if run_cb(&line.slice_with(range.clone())) == SplitCbRes::Stop {
                        return;
                    }
                    range.clear();
                    last_script = None;
                }
                let next = ((x / tab_width) + 1) * tab_width;
                range.shift('\t');
                for _ in x..next {
                    num_spaces += 1;
                }
                x = next;
            }
            c => {
                if num_spaces > 0 {
                    if space_cb(num_spaces) == SplitCbRes::Stop {
                        return;
                    }
                    num_spaces = 0;
                }
                let script_here = c.script();
                if script_here != Script::Unknown && script_here != Script::Common {
                    if let Some(script) = last_script {
                        if script != script_here {
                            if run_cb(&line.slice_with(range.clone())) == SplitCbRes::Stop {
                                return;
                            }
                            range.clear();
                        }
                    }
                    last_script = Some(script_here);
                }
                range.push(c);
                x += 1;
                // FIXME: Move x by graphemes?
            }
        }
    }
    if num_spaces > 0 {
        space_cb(num_spaces);
    } else if range.len() > 0 {
        run_cb(&line.slice_with(range.clone()));
    }
}

// Modified from https://github.com/cessen/ropey/blob/master/examples/graphemes_iter.rs
// An implementation of a graphemes iterator, for iterating over
// the graphemes of a RopeSlice.
pub(crate) struct RopeGraphemes<'a> {
    text: RopeSlice<'a>,
    chunks: Chunks<'a>,
    cur_chunk: &'a str,
    cur_chunk_start: usize,
    cursor: GraphemeCursor,
    idx: usize,
}

impl<'a> RopeGraphemes<'a> {
    pub(crate) fn new<'b>(slice: &RopeSlice<'b>) -> RopeGraphemes<'b> {
        let mut chunks = slice.chunks();
        let first_chunk = chunks.next().unwrap_or("");
        RopeGraphemes {
            text: *slice,
            chunks,
            cur_chunk: first_chunk,
            cur_chunk_start: 0,
            cursor: GraphemeCursor::new(0, slice.len_bytes(), true),
            idx: 0,
        }
    }
}

impl<'a> Iterator for RopeGraphemes<'a> {
    type Item = (usize, RopeSlice<'a>);

    fn next(&mut self) -> Option<(usize, RopeSlice<'a>)> {
        let a = self.cursor.cur_cursor();
        let idx = self.idx;
        let b;
        loop {
            match self
                .cursor
                .next_boundary(self.cur_chunk, self.cur_chunk_start)
            {
                Ok(None) => {
                    return None;
                }
                Ok(Some(n)) => {
                    b = n;
                    break;
                }
                Err(GraphemeIncomplete::NextChunk) => {
                    self.cur_chunk_start += self.cur_chunk.len();
                    self.cur_chunk = self.chunks.next().unwrap_or("");
                }
                _ => unreachable!(),
            }
        }
        self.idx += b - a;
        if a < self.cur_chunk_start {
            let a_char = self.text.byte_to_char(a);
            let b_char = self.text.byte_to_char(b);
            Some((idx, self.text.slice(a_char..b_char)))
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some((idx, (&self.cur_chunk[a2..b2]).into()))
        }
    }
}

// From https://github.com/cessen/ropey/blob/master/examples/graphemes_step.rs pub(crate)
pub(crate) fn rope_next_grapheme_boundary(slice: &RopeSlice, char_idx: usize) -> usize {
    // We work with bytes for this, so convert.
    let byte_idx = slice.char_to_byte(char_idx);
    // Get the chunk with our byte index in it.
    let (mut chunk, mut chunk_byte_idx, mut chunk_char_idx, _) = slice.chunk_at_byte(byte_idx);
    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);
    // Find the next grapheme cluster boundary.
    loop {
        match gc.next_boundary(chunk, chunk_byte_idx) {
            Ok(None) => return slice.len_chars(),
            Ok(Some(n)) => {
                let tmp = byte_to_char_idx(chunk, n - chunk_byte_idx);
                return chunk_char_idx + tmp;
            }
            Err(GraphemeIncomplete::NextChunk) => {
                chunk_byte_idx += chunk.len();
                let (a, _, c, _) = slice.chunk_at_byte(chunk_byte_idx);
                chunk = a;
                chunk_char_idx = c;
            }
            Err(GraphemeIncomplete::PreContext(n)) => {
                let ctx_chunk = slice.chunk_at_byte(n - 1).0;
                gc.provide_context(ctx_chunk, n - ctx_chunk.len());
            }
            _ => unreachable!(),
        }
    }
}

// From https://github.com/cessen/ropey/blob/master/examples/graphemes_step.rs
pub(crate) fn rope_is_grapheme_boundary(slice: &RopeSlice, char_idx: usize) -> bool {
    // We work with bytes for this, so convert.
    let byte_idx = slice.char_to_byte(char_idx);
    // Get the chunk with our byte index in it.
    let (chunk, chunk_byte_idx, _, _) = slice.chunk_at_byte(byte_idx);
    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);
    // Determine if the given position is a grapheme cluster boundary.
    loop {
        match gc.is_boundary(chunk, chunk_byte_idx) {
            Ok(n) => return n,
            Err(GraphemeIncomplete::PreContext(n)) => {
                let (ctx_chunk, ctx_byte_start, _, _) = slice.chunk_at_byte(n - 1);
                gc.provide_context(ctx_chunk, ctx_byte_start);
            }
            _ => unreachable!(),
        }
    }
}
