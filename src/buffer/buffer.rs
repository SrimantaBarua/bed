// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::File;
use std::io::Result as IOResult;
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use euclid::{Point2D, Rect, Vector2D};
use fnv::FnvHashMap;
use ropey::{Rope, RopeBuilder};
use syntect::highlighting::{
    FontStyle, HighlightState, Highlighter, RangedHighlightIterator, ThemeSet,
};
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};

use crate::common::{rope_trim_newlines, PixelSize};
use crate::painter::WidgetPainter;
use crate::style::{Color, TextSlant, TextStyle, TextWeight};

use super::hlpool::{HlPool, PARSE_CACHE_DIFF};
use super::view::{BufferView, BufferViewCreateParams, StyledText};
use super::{BufferID, BufferViewID};

pub(crate) struct Buffer {
    buf_id: BufferID,
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
    hl_states: Arc<Mutex<Vec<HighlightState>>>,
    parse_states: Arc<Mutex<Vec<ParseState>>>,
    styled_lines: Arc<Mutex<Vec<StyledText>>>,
    syntax_set: Arc<SyntaxSet>,
    theme_set: Arc<ThemeSet>,
    cur_theme: String,
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

    pub(crate) fn draw_view(&mut self, id: &BufferViewID, painter: &mut WidgetPainter) {
        self.views.get_mut(id).unwrap().draw(painter);
    }

    pub(crate) fn check_view_needs_redraw(&mut self, id: &BufferViewID) -> bool {
        let hlpool = &mut *self.hlpool.borrow_mut();
        if let Some(linum) = hlpool.highlight_checkpoint(self.buf_id) {
            let styled = self.styled_lines.lock().unwrap();
            for view in self.views.values_mut() {
                view.rehighlight_to(&self.data, &styled, linum);
            }
        }
        self.views.get(id).unwrap().needs_redraw
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
        let sync_upto = view.max_line_visible();
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
        self.rehighlight_from(linum, sync_upto);
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
        let sync_upto = view.max_line_visible();
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
        self.rehighlight_from(linum, sync_upto);
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
        let sync_upto = view.max_line_visible();
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
        self.rehighlight_from(linum, sync_upto);
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
            syntax_set: syntax_set,
            theme_set: theme_set,
            cur_theme: cur_theme.to_owned(),
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
        File::open(path).and_then(|f| {
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
                data: Rope::new(),
                views: FnvHashMap::default(),
                hl_states: Arc::new(Mutex::new(vec![hl_state])),
                parse_states: Arc::new(Mutex::new(vec![parse_state])),
                styled_lines: Arc::new(Mutex::new(Vec::new())),
                syntax_set: syntax_set,
                theme_set: theme_set,
                cur_theme: cur_theme.to_owned(),
                hlpool: hlpool,
            };
            ret.load_and_hl_file(f).map(|_| ret)
        })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|f| self.load_and_hl_file(f))
            .map(|_| {
                for view in self.views.values_mut() {
                    view.rehighlight_to(
                        &self.data,
                        &self.styled_lines.lock().unwrap(),
                        self.data.len_lines(),
                    );
                }
            })
    }

    fn rehighlight_from(&mut self, start_linum: usize, sync_upto: usize) {
        let valid_upto = (start_linum / PARSE_CACHE_DIFF) * PARSE_CACHE_DIFF;
        for view in self.views.values_mut() {
            view.hl_valid_upto = valid_upto;
        }
        let hlpool = &mut *self.hlpool.borrow_mut();
        hlpool.start_highlight(
            self.buf_id,
            self.data.clone(),
            Arc::clone(&self.parse_states),
            Arc::clone(&self.hl_states),
            Arc::clone(&self.styled_lines),
            start_linum,
            sync_upto,
        );
    }

    fn load_and_hl_file(&mut self, f: File) -> IOResult<()> {
        for view in self.views.values_mut() {
            view.hl_valid_upto = 0;
        }

        let mut reader = BufReader::new(f);
        let mut buf = String::new();
        let mut builder = RopeBuilder::new();

        let mut hl_states = self.hl_states.lock().unwrap();
        let mut parse_states = self.parse_states.lock().unwrap();
        let mut styled_lines = self.styled_lines.lock().unwrap();
        hl_states.truncate(1);
        parse_states.truncate(1);
        styled_lines.clear();

        let hl = Highlighter::new(self.theme_set.themes.get(&self.cur_theme).unwrap());
        let mut hlstate = hl_states[0].clone();
        let mut parse_state = parse_states[0].clone();
        let mut linum = 0;

        loop {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => {
                    self.data = builder.finish();
                    if self.data.len_lines() > styled_lines.len() {
                        let mut styled = StyledText::new();
                        styled.push(0, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
                        styled_lines.push(styled);
                        assert!(self.data.len_lines() == styled_lines.len());
                    }
                    break Ok(());
                }
                Ok(_) => {
                    builder.append(&buf);

                    let mut styled = StyledText::new();
                    let ops = parse_state.parse_line(&buf, &self.syntax_set);
                    for (style, txt, _) in
                        RangedHighlightIterator::new(&mut hlstate, &ops, &buf, &hl)
                    {
                        // TODO Background color
                        let clr = Color::from_syntect(style.foreground);
                        let mut ts = TextStyle::default();
                        if style.font_style.contains(FontStyle::BOLD) {
                            ts.weight = TextWeight::Bold;
                        }
                        if style.font_style.contains(FontStyle::ITALIC) {
                            ts.slant = TextSlant::Italic;
                        }
                        let under = if style.font_style.contains(FontStyle::UNDERLINE) {
                            Some(clr)
                        } else {
                            None
                        };
                        let ccount = txt.chars().count();
                        styled.push(ccount, ts, clr, under);
                    }
                    if styled.is_empty() {
                        styled.push(0, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
                    }
                    styled_lines.push(styled);
                    linum += 1;
                    if linum % PARSE_CACHE_DIFF == 0 {
                        hl_states.push(hlstate.clone());
                        parse_states.push(parse_state.clone());
                    }
                }
                Err(e) => break Err(e),
            }
        }
    }
}
