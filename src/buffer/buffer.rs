// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fs::File;
use std::io::Result as IOResult;
use std::rc::Rc;
use std::fmt::Write;

use euclid::{Point2D, Rect, Vector2D};
use fnv::FnvHashMap;
use ropey::Rope;
use syntect::highlighting::{ThemeSet, HighlightState, Highlighter, RangedHighlightIterator};
use syntect::parsing::{ParseState, SyntaxSet, ScopeStack};

use crate::common::PixelSize;
use crate::painter::WidgetPainter;

use super::view::{BufferView, BufferViewCreateParams};
use super::BufferViewID;

const PARSE_CACHE_DIFF: usize = 1000;

pub(crate) struct Buffer {
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
    syntax_set: Rc<SyntaxSet>,
    theme_set: Rc<ThemeSet>,
    hl_states: Vec<HighlightState>,
    parse_states: Vec<ParseState>,
    cur_theme: String,
    tab_width: usize,
}

impl Buffer {
    // -------- View management ----------------
    pub(crate) fn new_view(&mut self, id: &BufferViewID, params: BufferViewCreateParams) {
        self.views.insert(
            id.clone(),
            BufferView::new(params, &self.data, self.tab_width),
        );
    }

    pub(crate) fn set_view_rect(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views.get_mut(id).unwrap().set_rect(rect, &self.data);
    }

    pub(crate) fn draw_view(&self, id: &BufferViewID, painter: &mut WidgetPainter) {
        self.views.get(id).unwrap().draw(painter);
    }

    pub(crate) fn remove_view(&mut self, id: &BufferViewID) {
        self.views.remove(id);
    }

    pub(crate) fn scroll_view(&mut self, id: &BufferViewID, vec: Vector2D<i32, PixelSize>) {
        self.views.get_mut(id).unwrap().scroll(vec, &self.data);
    }

    // -------- View cursor motion ----------------
    pub(crate) fn move_view_cursor_up(&mut self, id: &BufferViewID, n: usize) {
        let view = self.views.get_mut(id).unwrap();
        if view.cursor.line_num == 0 {
            view.cursor.char_idx = 0;
            view.cursor.line_cidx = 0;
            view.cursor.line_gidx = 0;
            view.cursor.line_global_x = 0;
            return;
        }
        if view.cursor.line_num < n {
            view.cursor.line_num = 0;
        } else {
            view.cursor.line_num -= n;
        }
        view.cursor.sync_global_x(&self.data, self.tab_width);
        view.snap_to_cursor(&self.data);
    }

    pub(crate) fn move_view_cursor_down(&mut self, id: &BufferViewID, n: usize) {
        let view = self.views.get_mut(id).unwrap();
        view.cursor.line_num += n;
        if view.cursor.line_num >= self.data.len_lines() {
            view.cursor.char_idx = self.data.len_chars();
            view.cursor
                .sync_and_update_char_idx_left(&self.data, self.tab_width);
        } else {
            view.cursor.sync_global_x(&self.data, self.tab_width);
        }
        view.snap_to_cursor(&self.data);
    }

    pub(crate) fn move_view_cursor_left(&mut self, id: &BufferViewID, n: usize) {
        let view = self.views.get_mut(id).unwrap();
        if view.cursor.line_cidx <= n {
            view.cursor.char_idx -= view.cursor.line_cidx;
            view.cursor.line_cidx = 0;
            view.cursor.line_gidx = 0;
            view.cursor.line_global_x = 0;
        } else {
            view.cursor.line_cidx -= n;
            view.cursor
                .sync_line_cidx_gidx_left(&self.data, self.tab_width);
        }
        view.snap_to_cursor(&self.data);
    }

    pub(crate) fn move_view_cursor_right(&mut self, id: &BufferViewID, n: usize) {
        let view = self.views.get_mut(id).unwrap();
        view.cursor.line_cidx += n;
        view.cursor
            .sync_line_cidx_gidx_right(&self.data, self.tab_width);
        view.snap_to_cursor(&self.data);
    }

    pub(crate) fn move_view_cursor_to_point(
        &mut self,
        id: &BufferViewID,
        point: Point2D<u32, PixelSize>,
    ) {
        let view = self.views.get_mut(id).unwrap();
        view.move_cursor_to_point(point, &self.data, self.tab_width);
    }

    // -------- View edits -----------------
    pub(crate) fn view_insert_char(&mut self, id: &BufferViewID, c: char) {
        let view = self.views.get_mut(id).unwrap();
        let cidx = view.cursor.char_idx;
        let linum = view.cursor.line_num;
        let mut end_linum = linum;
        match c {
            // Insert pair
            '[' | '{' | '(' => {
                self.data.insert_char(cidx, c);
                if cidx + 1 == self.data.len_chars() || self.data.char(cidx + 1).is_whitespace() {
                    match c {
                        '[' => self.data.insert_char(cidx + 1, ']'),
                        '{' => self.data.insert_char(cidx + 1, '}'),
                        '(' => self.data.insert_char(cidx + 1, ')'),
                        _ => unreachable!(),
                    }
                }
            }
            // Maybe insert pair, maybe skip
            '"' | '\'' => {
                if self.data.char(cidx) != c {
                    self.data.insert_char(cidx, c);
                    self.data.insert_char(cidx + 1, c);
                } else {
                    return self.move_view_cursor_right(id, 1);
                }
            }
            // Maybe skip insert
            ']' | '}' | ')' => {
                if self.data.char(cidx) != c {
                    self.data.insert_char(cidx, c);
                } else {
                    return self.move_view_cursor_right(id, 1);
                }
            }
            // Maybe insert twice?
            '\n' | ' ' => {
                self.data.insert_char(cidx, c);
                if c == '\n' {
                    end_linum += 1;
                }
                if cidx > 0 && cidx + 1 < self.data.len_chars() {
                    let c0 = self.data.char(cidx - 1);
                    let c1 = self.data.char(cidx + 1);
                    if (c0 == '(' && c1 == ')')
                        || (c0 == '{' && c1 == '}')
                        || (c0 == '[' && c1 == ']')
                    {
                        self.data.insert_char(cidx + 1, c);
                        if c == '\n' {
                            end_linum += 1;
                        }
                    }
                }
            }
            c => self.data.insert_char(cidx, c),
        }
        for view in self.views.values_mut() {
            if view.cursor.char_idx >= cidx {
                view.cursor.char_idx += 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, linum);
            for i in linum..end_linum {
                view.insert_line(&self.data, i + 1);
            }
            view.snap_to_cursor(&self.data);
        }
    }

    pub(crate) fn view_delete_left(&mut self, id: &BufferViewID) {
        let view = self.views.get_mut(id).unwrap();
        if view.cursor.char_idx == 0 {
            return;
        }
        let cidx = view.cursor.char_idx;
        let len_lines = self.data.len_lines();
        self.data.remove(cidx - 1..cidx);
        let mut linum = view.cursor.line_num;
        let is_beg = view.cursor.line_cidx == 0 && self.data.len_lines() < len_lines;
        if is_beg {
            linum -= 1;
        }
        for view in self.views.values_mut() {
            if view.cursor.char_idx >= cidx {
                view.cursor.char_idx -= 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, linum);
            if is_beg {
                view.delete_line(&self.data, linum + 1);
            }
            view.snap_to_cursor(&self.data);
        }
    }

    pub(crate) fn view_delete_right(&mut self, id: &BufferViewID) {
        let view = self.views.get_mut(id).unwrap();
        if view.cursor.char_idx == self.data.len_chars() {
            return;
        }
        let cidx = view.cursor.char_idx;
        let len_lines = self.data.len_lines();
        self.data.remove(cidx..cidx + 1);
        let linum = view.cursor.line_num;
        let del_end = self.data.len_lines() < len_lines;
        for view in self.views.values_mut() {
            if view.cursor.char_idx > cidx {
                view.cursor.char_idx -= 1;
            }
            if view.cursor.char_idx >= cidx {
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, linum);
            if del_end {
                view.delete_line(&self.data, linum + 1);
            }
            view.snap_to_cursor(&self.data);
        }
    }

    // -------- Create buffer ----------------
    pub(super) fn empty(
        syntax_set: Rc<SyntaxSet>,
        theme_set: Rc<ThemeSet>,
        cur_theme: &str,
    ) -> Buffer {
        let synref = syntax_set.find_syntax_plain_text();
        let hl = Highlighter::new(theme_set.themes.get(cur_theme).unwrap());
        let hl_state = HighlightState::new(&hl, ScopeStack::new());
        let parse_state = ParseState::new(synref);
        Buffer {
            tab_width: 8,
            data: Rope::new(),
            views: FnvHashMap::default(),
            syntax_set: syntax_set,
            theme_set: theme_set,
            cur_theme: cur_theme.to_owned(),
            hl_states: vec![hl_state],
            parse_states: vec![parse_state],
        }
    }

    pub(super) fn from_file(
        path: &str,
        syntax_set: Rc<SyntaxSet>,
        theme_set: Rc<ThemeSet>,
        cur_theme: &str,
    ) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| {
                let synref = syntax_set
                    .find_syntax_for_file(path)
                    .unwrap()
                    .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                let parse_state = ParseState::new(synref);
                let hl = Highlighter::new(theme_set.themes.get(cur_theme).unwrap());
                let hl_state = HighlightState::new(&hl, ScopeStack::new());
                let mut ret = Buffer {
                    tab_width: 8,
                    data: rope,
                    views: FnvHashMap::default(),
                    syntax_set: syntax_set,
                    theme_set: theme_set,
                    hl_states: vec![hl_state],
                    parse_states: vec![parse_state],
                    cur_theme: cur_theme.to_owned(),
                };
                ret
            })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| self.data = rope)
    }

    fn rehighlight_from(&mut self, mut linum: usize) {
        let i = linum / PARSE_CACHE_DIFF;
        self.hl_states.truncate(i + 1);
        self.parse_states.truncate(i + 1);
        let mut buf = String::new();
        let hl = Highlighter::new(self.theme_set.themes.get(&self.cur_theme).unwrap());
        let mut hlstate = self.hl_states[i].clone();
        let mut parse_state = self.parse_states[i].clone();
        for line in self.data.lines_at(linum) {
            buf.clear();
            write!(&mut buf, "{}", line).unwrap();
            let ops = parse_state.parse_line(&buf, &self.syntax_set);
            for (style, txt, _) in RangedHighlightIterator::new(&mut hlstate, &ops, &buf, &hl) {
            }
            linum += 1;
            if linum % PARSE_CACHE_DIFF == 0 {
                self.hl_states.push(hlstate.clone());
                self.parse_states.push(parse_state.clone());
            }
        }
    }
}
