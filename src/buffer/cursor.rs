// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use ropey::{Rope, RopeSlice};

use crate::common::{
    rope_is_grapheme_boundary, rope_next_grapheme_boundary, rope_trim_newlines, RopeGraphemes,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum CursorStyle {
    Line,
    Underline,
    Block,
}

pub(super) struct Cursor {
    pub(super) visible: bool,
    pub(super) char_idx: usize,
    pub(super) line_num: usize,
    pub(super) line_cidx: usize,
    pub(super) line_gidx: usize,
    pub(super) line_global_x: usize,
    pub(super) style: CursorStyle,
}

impl Cursor {
    pub(super) fn sync_and_update_char_idx_left(&mut self, data: &Rope, tab_width: usize) {
        self.line_num = data.char_to_line(self.char_idx);
        self.line_cidx = self.char_idx - data.line_to_char(self.line_num);
        self.sync_line_cidx_gidx_left(data, tab_width);
    }

    /*
    pub(super) fn sync_and_update_char_idx_right(&mut self, data: &Rope, tab_width: usize) {
        self.line_num = data.char_to_line(self.char_idx);
        self.line_cidx = self.char_idx - data.line_to_char(self.line_num);
        self.sync_line_cidx_gidx_right(data, tab_width);
    }
    */

    pub(super) fn sync_line_cidx_gidx_left(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let len_chars = trimmed.len_chars();
        if self.line_cidx >= len_chars {
            self.line_cidx = len_chars;
            if !self.past_end() && self.line_cidx > 0 {
                self.line_cidx -= 1;
            }
        }
        let (cidx, gidx) = cidx_gidx_from_cidx(&trimmed, self.line_cidx, tab_width);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.char_idx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn sync_line_cidx_gidx_right(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let len_chars = trimmed.len_chars();
        if self.line_cidx > len_chars {
            self.line_cidx = len_chars;
        }
        if !rope_is_grapheme_boundary(&trimmed, self.line_cidx) {
            self.line_cidx = rope_next_grapheme_boundary(&trimmed, self.line_cidx);
        }
        if !self.past_end() && self.line_cidx == len_chars && self.line_cidx > 0 {
            self.line_cidx -= 1;
        }
        let (cidx, gidx) = cidx_gidx_from_cidx(&trimmed, self.line_cidx, tab_width);
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.char_idx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn sync_global_x(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let (cidx, gidx) =
            cidx_gidx_from_global_x(&trimmed, self.line_global_x, tab_width, self.past_end());
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.char_idx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn sync_gidx(&mut self, data: &Rope, tab_width: usize) {
        let trimmed = rope_trim_newlines(data.line(self.line_num));
        let (cidx, gidx) =
            cidx_gidx_from_gidx(&trimmed, self.line_gidx, tab_width, self.past_end());
        self.line_cidx = cidx;
        self.line_gidx = gidx;
        self.line_global_x = self.line_gidx;
        self.char_idx = data.line_to_char(self.line_num) + self.line_cidx;
    }

    pub(super) fn default() -> Cursor {
        Cursor {
            visible: true,
            char_idx: 0,
            line_num: 0,
            line_cidx: 0,
            line_gidx: 0,
            line_global_x: 0,
            style: CursorStyle::Block,
        }
    }

    pub(super) fn past_end(&self) -> bool {
        self.style == CursorStyle::Line
    }
}

/*
fn gidx_from_cidx(line: &RopeSlice, cidx: usize, tab_width: usize) -> usize {
    let (mut gidx, mut ccount) = (0, 0);
    for g in RopeGraphemes::new(line) {
        ccount += g.chars().count();
        if ccount > cidx {
            return gidx;
        }
        if g == "\t" {
            gidx = (gidx / tab_width) * tab_width + tab_width;
        } else {
            gidx += 1;
        }
    }
    gidx
}
*/

fn cidx_gidx_from_cidx(slice: &RopeSlice, cidx: usize, tab_width: usize) -> (usize, usize) {
    let (mut gidx, mut ccount) = (0, 0);
    for g in RopeGraphemes::new(slice) {
        let count_here = g.chars().count();
        if ccount + count_here > cidx {
            return (ccount, gidx);
        }
        ccount += count_here;
        if g == "\t" {
            gidx = (gidx / tab_width) * tab_width + tab_width;
        } else {
            gidx += 1;
        }
    }
    (ccount, gidx)
}

pub(super) fn cidx_gidx_from_gidx(
    slice: &RopeSlice,
    gidx: usize,
    tab_width: usize,
    past_end: bool,
) -> (usize, usize) {
    let (mut gcount, mut cidx) = (0, 0);
    let mut len_chars = slice.len_chars();
    if !past_end && len_chars > 0 {
        len_chars -= 1;
    }
    for g in RopeGraphemes::new(slice) {
        let count_here = g.chars().count();
        if gcount >= gidx || cidx + count_here > len_chars {
            return (cidx, gcount);
        }
        cidx += count_here;
        if g == "\t" {
            gcount = (gcount / tab_width) * tab_width + tab_width;
        } else {
            gcount += 1;
        }
    }
    (cidx, gcount)
}

fn cidx_gidx_from_global_x(
    slice: &RopeSlice,
    global_x: usize,
    tab_width: usize,
    past_end: bool,
) -> (usize, usize) {
    let (mut gidx, mut ccount) = (0, 0);
    let mut len_chars = slice.len_chars();
    if !past_end && len_chars > 0 {
        len_chars -= 1;
    }
    for g in RopeGraphemes::new(slice) {
        let count_here = g.chars().count();
        if gidx >= global_x || ccount + count_here > len_chars {
            return (ccount, gidx);
        }
        ccount += count_here;
        if g == "\t" {
            gidx = (gidx / tab_width) * tab_width + tab_width;
        } else {
            gidx += 1;
        }
    }
    (ccount, gidx)
}
