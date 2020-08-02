// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use ropey::{iter::Chunks, str_utils::byte_to_char_idx, RopeSlice};
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

// -------- Dummy data types --------
pub(crate) struct PixelSize;
pub(crate) struct TextureSize;

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

// Split text into runs and spaces
pub(crate) fn split_text<S, R>(
    line: &RopeSlice, // FIXME: Use RopeOrStr
    tab_width: usize,
    mut space_cb: S,
    mut run_cb: R,
) where
    S: FnMut(usize),
    R: FnMut(&str),
{
    let mut buf = String::new();
    let mut last_is_space = false;
    let mut last_script = None;
    let mut x = 0;
    for c in line.chars() {
        match c {
            '\n' | '\r' | '\x0b' | '\x0c' | '\u{85}' | '\u{2028}' | '\u{2029}' => break,
            ' ' => {
                if !last_is_space && buf.len() > 0 {
                    run_cb(&buf);
                    buf.clear();
                    last_script = None;
                }
                last_is_space = true;
                x += 1;
                buf.push(c);
            }
            '\t' => {
                if !last_is_space && buf.len() > 0 {
                    run_cb(&buf);
                    buf.clear();
                    last_script = None;
                }
                last_is_space = true;
                let next = ((x / tab_width) + 1) * tab_width;
                for _ in x..next {
                    buf.push(' ');
                }
                x = next;
            }
            c => {
                if last_is_space && buf.len() > 0 {
                    space_cb(buf.len());
                    buf.clear();
                }
                let script_here = c.script();
                if script_here != Script::Unknown && script_here != Script::Common {
                    if let Some(script) = last_script {
                        if script != script_here {
                            run_cb(&buf);
                            buf.clear();
                        }
                    }
                    last_script = Some(script_here);
                }
                last_is_space = false;
                buf.push(c);
                x += 1;
                // FIXME: Move x by graphemes
            }
        }
    }
    if buf.len() > 0 {
        if last_is_space {
            space_cb(buf.len());
        } else {
            run_cb(&buf);
        }
    }
}

// From https://github.com/cessen/ropey/blob/master/examples/graphemes_iter.rs
// An implementation of a graphemes iterator, for iterating over
// the graphemes of a RopeSlice.
pub(crate) struct RopeGraphemes<'a> {
    text: RopeSlice<'a>,
    chunks: Chunks<'a>,
    cur_chunk: &'a str,
    cur_chunk_start: usize,
    cursor: GraphemeCursor,
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
        }
    }
}

impl<'a> Iterator for RopeGraphemes<'a> {
    type Item = RopeSlice<'a>;

    fn next(&mut self) -> Option<RopeSlice<'a>> {
        let a = self.cursor.cur_cursor();
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

        if a < self.cur_chunk_start {
            let a_char = self.text.byte_to_char(a);
            let b_char = self.text.byte_to_char(b);

            Some(self.text.slice(a_char..b_char))
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some((&self.cur_chunk[a2..b2]).into())
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
