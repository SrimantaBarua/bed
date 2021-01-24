// (C) 2021 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::min;
use std::ops::Range;

use crate::common::{is_ropey_newline, rope_trim_newlines};
use crate::input::MoveObj;

use super::{Buffer, BufferViewId};

impl Buffer {
    // -------- Editing --------
    pub(crate) fn insert(&mut self, view_id: &BufferViewId, s: &str) {
        let (old_rope, cidx, num_chars) = {
            let mut shared = self.shared.borrow_mut();
            let old_rope = shared.rope.clone();
            let view = self.views.get_mut(view_id).unwrap();
            let cidx = view.cursor.cidx;
            let mut num_chars = 0;
            let mut x = view.cursor.line_cidx;
            for c in s.chars() {
                match c {
                    c if is_ropey_newline(c) => {
                        x = 0;
                        shared.rope.insert_char(cidx + num_chars, '\n');
                        num_chars += 1;
                    }
                    '\t' => {
                        let next_stop = ((x / shared.tab_width) + 1) * shared.tab_width;
                        if shared.indent_tabs {
                            shared.rope.insert_char(cidx + num_chars, '\t');
                            num_chars += 1;
                        } else {
                            shared.rope.insert(cidx + num_chars, &" ".repeat(num_chars));
                            num_chars += next_stop - x;
                        }
                        x = next_stop;
                    }
                    _ => {
                        shared.rope.insert_char(cidx + num_chars, c);
                        num_chars += 1;
                        x += 1;
                    }
                }
            }
            let fgcol = self.bed_handle.theme().textview.foreground;
            shared.styles.insert_default(cidx, num_chars, fgcol);
            for view in self.views.values_mut() {
                if view.cursor.cidx >= cidx {
                    view.cursor.cidx += num_chars;
                    view.cursor
                        .sync_and_update_char_idx_left(&shared.rope, shared.tab_width);
                }
            }
            (old_rope, cidx, num_chars)
        };
        self.edit_tree(old_rope, cidx..cidx, num_chars);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn delete(&mut self, view_id: &BufferViewId, move_obj: MoveObj) -> String {
        match move_obj {
            MoveObj::Left(n) => self.delete_left(view_id, n),
            MoveObj::Right(n) => self.delete_right(view_id, n),
            MoveObj::Up(n) => self.delete_up(view_id, n),
            MoveObj::Down(n) => self.delete_down(view_id, n),
            MoveObj::ToLine(linum) => self.delete_to_line(view_id, linum),
            MoveObj::ToLastLine => self.delete_to_last_line(view_id),
            //MoveObj::ToViewFirstLine => self.delete_to_view_first_line(view_id),
            //MoveObj::ToViewLastLine => self.delete_to_view_last_line(view_id),
            //MoveObj::ToViewMiddleLine => self.delete_to_view_middle_line(view_id),
            MoveObj::LineStart(n) => self.delete_to_line_start(view_id, n),
            MoveObj::LineEnd(n) => self.delete_to_line_end(view_id, n),
            MoveObj::LineFirstNonBlank => self.delete_to_first_non_blank(view_id),
            MoveObj::WordBeg(n, false) => self.delete_word(view_id, n),
            MoveObj::WordBeg(n, true) => self.delete_word_extended(view_id, n),
            MoveObj::WordEnd(n, false) => self.delete_word_end(view_id, n),
            MoveObj::WordEnd(n, true) => self.delete_word_end_extended(view_id, n),
            MoveObj::Back(n, false) => self.delete_back(view_id, n),
            MoveObj::Back(n, true) => self.delete_back_extended(view_id, n),
            _ => unimplemented!(),
        }
    }

    fn delete_range(&mut self, range: Range<usize>) -> String {
        let (old_rope, range, ret_s) = {
            let mut shared = self.shared.borrow_mut();
            let old_rope = shared.rope.clone();
            let ret_s = shared.rope.slice(range.clone()).to_string();
            shared.rope.remove(range.clone());
            shared.styles.remove(range.clone());
            for view in self.views.values_mut() {
                if view.cursor.cidx >= range.end {
                    view.cursor.cidx -= range.len();
                } else if view.cursor.cidx > range.start {
                    view.cursor.cidx = range.start;
                }
                if view.cursor.cidx >= range.start {
                    view.cursor
                        .sync_and_update_char_idx_left(&shared.rope, shared.tab_width);
                }
            }
            (old_rope, range, ret_s)
        };
        self.edit_tree(old_rope, range, 0);
        self.bed_handle.request_redraw();
        ret_s
    }

    fn delete_left(&mut self, view_id: &BufferViewId, mut n: usize) -> String {
        let view = self.views.get(view_id).unwrap();
        let cidx = view.cursor.cidx;
        if cidx < n {
            n = cidx;
        }
        let start_cidx = cidx - n;
        self.delete_range(start_cidx..cidx)
    }

    fn delete_right(&mut self, view_id: &BufferViewId, mut n: usize) -> String {
        let view = self.views.get(view_id).unwrap();
        let mut cidx = view.cursor.cidx;
        let pre_len_chars = self.shared.borrow().rope.len_chars();
        if pre_len_chars == 0 {
            return "".to_owned();
        }
        assert!(cidx <= pre_len_chars);
        if cidx == pre_len_chars {
            cidx = pre_len_chars - 1;
            n = 1;
        } else if cidx + n >= pre_len_chars {
            n = pre_len_chars - cidx;
        }
        let end_cidx = cidx + n;
        self.delete_range(cidx..end_cidx)
    }

    fn delete_up(&mut self, view_id: &BufferViewId, mut n: usize) -> String {
        let view = self.views.get(view_id).unwrap();
        if view.cursor.line_num < n {
            n = view.cursor.line_num;
        }
        self.move_cursor_up(view_id, n);
        self.delete_down(view_id, n)
    }

    fn delete_down(&mut self, view_id: &BufferViewId, mut n: usize) -> String {
        let view = self.views.get(view_id).unwrap();
        let range = {
            let shared = self.shared.borrow();
            if view.cursor.line_num + n > shared.rope.len_lines() {
                n = shared.rope.len_lines() - view.cursor.line_num;
            }
            let start_cidx = shared.rope.line_to_char(view.cursor.line_num);
            let end_cidx = if view.cursor.line_num + n == shared.rope.len_lines() {
                shared.rope.len_chars()
            } else {
                shared.rope.line_to_char(view.cursor.line_num + n + 1)
            };
            start_cidx..end_cidx
        };
        self.delete_range(range)
    }

    fn delete_to_line_start(&mut self, view_id: &BufferViewId, n: usize) -> String {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_line_start(&view.cursor, n);
            dest.cidx..view.cursor.cidx
        };
        self.delete_range(range)
    }

    fn delete_to_line_end(&mut self, view_id: &BufferViewId, n: usize) -> String {
        let range = {
            let shared = self.shared.borrow();
            let view = self.view(view_id);
            let dest = self.cursor_line_end(&view.cursor, n);
            view.cursor.cidx..min(dest.cidx, shared.rope.len_chars())
        };
        self.delete_range(range)
    }

    fn delete_to_first_non_blank(&mut self, view_id: &BufferViewId) -> String {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_first_non_blank(&view.cursor);
            dest.cidx..view.cursor.cidx
        };
        self.delete_range(range)
    }

    fn delete_to_line(&mut self, view_id: &BufferViewId, linum: usize) -> String {
        let view = self.view(view_id);
        if view.cursor.line_num < linum {
            let diff = linum - view.cursor.line_num;
            self.delete_down(view_id, diff)
        } else {
            let diff = view.cursor.line_num - linum;
            self.delete_up(view_id, diff)
        }
    }

    fn delete_to_last_line(&mut self, view_id: &BufferViewId) -> String {
        let len_lines = self.shared.borrow().rope.len_lines();
        self.delete_down(view_id, len_lines - self.view(view_id).cursor.line_num)
    }

    fn delete_word(&mut self, view_id: &BufferViewId, n: usize) -> String {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_word(&view.cursor, n, false);
            view.cursor.cidx..dest.cidx
        };
        self.delete_range(range)
    }

    fn delete_word_extended(&mut self, view_id: &BufferViewId, n: usize) -> String {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_word(&view.cursor, n, true);
            view.cursor.cidx..dest.cidx
        };
        self.delete_range(range)
    }

    fn delete_word_end(&mut self, view_id: &BufferViewId, n: usize) -> String {
        let range = {
            let shared = self.shared.borrow();
            let view = self.view(view_id);
            let dest = self.cursor_word_end(&view.cursor, n, false);
            view.cursor.cidx..min(dest.cidx + 1, shared.rope.len_chars())
        };
        self.delete_range(range)
    }

    fn delete_word_end_extended(&mut self, view_id: &BufferViewId, n: usize) -> String {
        let range = {
            let shared = self.shared.borrow();
            let view = self.view(view_id);
            let dest = self.cursor_word_end(&view.cursor, n, true);
            view.cursor.cidx..min(dest.cidx + 1, shared.rope.len_chars())
        };
        self.delete_range(range)
    }

    fn delete_back(&mut self, view_id: &BufferViewId, n: usize) -> String {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_back(&view.cursor, n, false);
            dest.cidx..view.cursor.cidx
        };
        self.delete_range(range)
    }

    fn delete_back_extended(&mut self, view_id: &BufferViewId, n: usize) -> String {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_back(&view.cursor, n, true);
            dest.cidx..view.cursor.cidx
        };
        self.delete_range(range)
    }

    // -------- Misc updates --------
    pub(crate) fn replace_repeated(&mut self, view_id: &BufferViewId, mut c: char, n: usize) {
        assert!(n > 0);
        if is_ropey_newline(c) {
            c = '\n';
        }
        let mut cursor = self.view(view_id).cursor.clone();
        let range = cursor.cidx..cursor.cidx + n;
        let (old_rope, cursor) = {
            let mut shared = self.shared.borrow_mut();
            let len_chars = rope_trim_newlines(shared.rope.line(cursor.line_num)).len_chars();
            if cursor.line_cidx + n > len_chars {
                return;
            }
            let mut buf = [0; 4];
            let s = c.encode_utf8(&mut buf);
            let old_rope = shared.rope.clone();
            shared.rope.remove(range.clone());
            shared.rope.insert(cursor.cidx, &s.repeat(n));
            let fgcol = self.bed_handle.theme().textview.foreground;
            shared.styles.set_default(range.clone(), fgcol);
            cursor.cidx += n - 1;
            cursor.sync_and_update_char_idx_left(&shared.rope, shared.tab_width);
            (old_rope, cursor)
        };
        self.edit_tree(old_rope, range, n);
        self.view_mut(view_id).cursor = cursor;
    }

    pub(crate) fn update_text_size(&mut self, view_id: &BufferViewId, diff: i16) {
        self.view_mut(view_id).update_text_size(diff);
        self.bed_handle.request_redraw();
    }
}
