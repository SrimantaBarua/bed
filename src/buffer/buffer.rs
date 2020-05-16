// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::File;
use std::io::Result as IOResult;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use euclid::{Point2D, Rect, Vector2D};
use fnv::FnvHashMap;
use ropey::Rope;
use syntect::highlighting::{HighlightState, Highlighter, ThemeSet};
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};

use crate::common::{rope_trim_newlines, PixelSize};
use crate::painter::WidgetPainter;
use crate::style::{Color, TextStyle};

use super::hlpool::HlPool;
use super::view::{BufferView, BufferViewCreateParams, StyledText};
use super::{BufferID, BufferViewID};

pub(crate) struct Buffer {
    buf_id: BufferID,
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
    hl_states: Arc<Mutex<Vec<HighlightState>>>,
    parse_states: Arc<Mutex<Vec<ParseState>>>,
    styled_lines: Arc<Mutex<Vec<StyledText>>>,
    tab_width: usize,
    hlpool: Rc<RefCell<HlPool>>,
}

impl Buffer {
    // -------- View management ----------------
    pub(crate) fn new_view(&mut self, id: &BufferViewID, params: BufferViewCreateParams) {
        self.views.insert(
            id.clone(),
            BufferView::new(
                params,
                &self.data,
                &self.styled_lines.lock().unwrap(),
                self.tab_width,
            ),
        );
    }

    pub(crate) fn set_view_rect(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views.get_mut(id).unwrap().set_rect(
            rect,
            &self.data,
            &self.styled_lines.lock().unwrap(),
        );
    }

    pub(crate) fn draw_view(&self, id: &BufferViewID, painter: &mut WidgetPainter) {
        self.views.get(id).unwrap().draw(painter);
    }

    pub(crate) fn remove_view(&mut self, id: &BufferViewID) {
        self.views.remove(id);
    }

    pub(crate) fn scroll_view(&mut self, id: &BufferViewID, vec: Vector2D<i32, PixelSize>) {
        self.views
            .get_mut(id)
            .unwrap()
            .scroll(vec, &self.data, &self.styled_lines.lock().unwrap());
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
        view.snap_to_cursor(&self.data, &self.styled_lines.lock().unwrap());
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
        view.snap_to_cursor(&self.data, &self.styled_lines.lock().unwrap());
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
        view.snap_to_cursor(&self.data, &self.styled_lines.lock().unwrap());
    }

    pub(crate) fn move_view_cursor_right(&mut self, id: &BufferViewID, n: usize) {
        let view = self.views.get_mut(id).unwrap();
        view.cursor.line_cidx += n;
        view.cursor
            .sync_line_cidx_gidx_right(&self.data, self.tab_width);
        view.snap_to_cursor(&self.data, &self.styled_lines.lock().unwrap());
    }

    pub(crate) fn move_view_cursor_to_point(
        &mut self,
        id: &BufferViewID,
        point: Point2D<u32, PixelSize>,
    ) {
        let view = self.views.get_mut(id).unwrap();
        view.move_cursor_to_point(
            point,
            &self.data,
            &self.styled_lines.lock().unwrap(),
            self.tab_width,
        );
    }

    // -------- View edits -----------------
    pub(crate) fn view_insert_char(&mut self, id: &BufferViewID, c: char) {
        {
            let hlpool = &mut *self.hlpool.borrow_mut();
            hlpool.stop_highlight(self.buf_id);
        }
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
        {
            let mut styled_lines = self.styled_lines.lock().unwrap();
            let lch = rope_trim_newlines(self.data.line(linum)).len_chars();
            let mut styled = StyledText::new();
            styled.push(lch, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
            styled_lines[linum] = styled;
            for i in linum..end_linum {
                let lch = rope_trim_newlines(self.data.line(i + 1)).len_chars();
                let mut styled = StyledText::new();
                styled.push(lch, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
                styled_lines.insert(i + 1, styled);
            }
        }
        self.rehighlight_from(linum);
        for view in self.views.values_mut() {
            if view.cursor.char_idx >= cidx {
                view.cursor.char_idx += 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, &self.styled_lines.lock().unwrap(), linum);
            for i in linum..end_linum {
                view.insert_line(&self.data, &self.styled_lines.lock().unwrap(), i + 1);
            }
            view.snap_to_cursor(&self.data, &self.styled_lines.lock().unwrap());
        }
    }

    pub(crate) fn view_delete_left(&mut self, id: &BufferViewID) {
        let view = self.views.get_mut(id).unwrap();
        if view.cursor.char_idx == 0 {
            return;
        }
        {
            let hlpool = &mut *self.hlpool.borrow_mut();
            hlpool.stop_highlight(self.buf_id);
        }
        let cidx = view.cursor.char_idx;
        let len_lines = self.data.len_lines();
        self.data.remove(cidx - 1..cidx);
        let mut linum = view.cursor.line_num;
        let is_beg = view.cursor.line_cidx == 0 && self.data.len_lines() < len_lines;
        {
            let mut styled_lines = self.styled_lines.lock().unwrap();
            if is_beg {
                styled_lines.remove(linum);
                linum -= 1;
            }
            let lch = rope_trim_newlines(self.data.line(linum)).len_chars();
            let mut styled = StyledText::new();
            styled.push(lch, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
            styled_lines[linum] = styled;
        }
        self.rehighlight_from(linum);
        for view in self.views.values_mut() {
            if view.cursor.char_idx >= cidx {
                view.cursor.char_idx -= 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, &self.styled_lines.lock().unwrap(), linum);
            if is_beg {
                view.delete_line(&self.data, &self.styled_lines.lock().unwrap(), linum + 1);
            }
            view.snap_to_cursor(&self.data, &self.styled_lines.lock().unwrap());
        }
    }

    pub(crate) fn view_delete_right(&mut self, id: &BufferViewID) {
        let view = self.views.get_mut(id).unwrap();
        if view.cursor.char_idx == self.data.len_chars() {
            return;
        }
        {
            let hlpool = &mut *self.hlpool.borrow_mut();
            hlpool.stop_highlight(self.buf_id);
        }
        let cidx = view.cursor.char_idx;
        let linum = view.cursor.line_num;
        let len_lines = self.data.len_lines();
        self.data.remove(cidx..cidx + 1);
        let del_end = self.data.len_lines() < len_lines;
        {
            let mut styled_lines = self.styled_lines.lock().unwrap();
            if del_end {
                styled_lines.remove(linum + 1);
            }
            let lch = rope_trim_newlines(self.data.line(linum)).len_chars();
            let mut styled = StyledText::new();
            styled.push(lch, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
            styled_lines[linum] = styled;
        }
        self.rehighlight_from(linum);
        for view in self.views.values_mut() {
            if view.cursor.char_idx > cidx {
                view.cursor.char_idx -= 1;
            }
            if view.cursor.char_idx >= cidx {
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, &self.styled_lines.lock().unwrap(), linum);
            if del_end {
                view.delete_line(&self.data, &self.styled_lines.lock().unwrap(), linum + 1);
            }
            view.snap_to_cursor(&self.data, &self.styled_lines.lock().unwrap());
        }
    }

    // -------- Create buffer ----------------
    pub(super) fn empty(
        buf_id: BufferID,
        syntax_set: Arc<SyntaxSet>,
        theme_set: Arc<ThemeSet>,
        cur_theme: &str,
        hlpool: Rc<RefCell<HlPool>>,
    ) -> Buffer {
        let synref = syntax_set.find_syntax_plain_text();
        let hl = Highlighter::new(theme_set.themes.get(cur_theme).unwrap());
        let hl_state = HighlightState::new(&hl, ScopeStack::new());
        let parse_state = ParseState::new(synref);
        let mut styled = StyledText::new();
        styled.push(0, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
        Buffer {
            buf_id: buf_id,
            tab_width: 8,
            data: Rope::new(),
            views: FnvHashMap::default(),
            hl_states: Arc::new(Mutex::new(vec![hl_state])),
            parse_states: Arc::new(Mutex::new(vec![parse_state])),
            styled_lines: Arc::new(Mutex::new(vec![styled])),
            hlpool: hlpool,
        }
    }

    pub(super) fn from_file(
        buf_id: BufferID,
        path: &str,
        syntax_set: Arc<SyntaxSet>,
        theme_set: Arc<ThemeSet>,
        cur_theme: &str,
        hlpool: Rc<RefCell<HlPool>>,
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
                    buf_id: buf_id,
                    tab_width: 8,
                    data: rope,
                    views: FnvHashMap::default(),
                    hl_states: Arc::new(Mutex::new(vec![hl_state])),
                    parse_states: Arc::new(Mutex::new(vec![parse_state])),
                    styled_lines: Arc::new(Mutex::new(Vec::new())),
                    hlpool: hlpool,
                };
                ret.refill_dummy_styled_lines();
                ret.rehighlight_from(0);
                ret
            })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| {
                {
                    let hlpool = &mut *self.hlpool.borrow_mut();
                    hlpool.stop_highlight(self.buf_id);
                }
                self.data = rope;
                self.refill_dummy_styled_lines();
                self.rehighlight_from(0);
            })
    }

    fn refill_dummy_styled_lines(&mut self) {
        let mut styled_lines = self.styled_lines.lock().unwrap();
        styled_lines.clear();
        for line in self.data.lines() {
            let lc = rope_trim_newlines(line).len_chars();
            let mut styled = StyledText::new();
            styled.push(lc, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
            styled_lines.push(styled);
        }
    }

    fn rehighlight_from(&mut self, linum: usize) {
        let hlpool = &mut *self.hlpool.borrow_mut();
        hlpool.start_highlight(
            self.buf_id,
            self.data.clone(),
            Arc::clone(&self.parse_states),
            Arc::clone(&self.hl_states),
            Arc::clone(&self.styled_lines),
            linum,
        );
    }
}
