// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::min;
use std::fs::File;
use std::io::Result as IOResult;
use std::path::Path;
use std::rc::Rc;

use euclid::{Point2D, Rect, Vector2D};
use fnv::FnvHashMap;
use ropey::{Rope, RopeSlice};
use tree_sitter::{InputEdit, Parser, Point, Query, QueryCursor, Tree};

use crate::common::{rope_trim_newlines, PixelSize};
use crate::input::Motion;
use crate::painter::Painter;
use crate::style::{Color, TextStyle};
use crate::theme::Theme;
use crate::ts::TsCore;

use super::styled::StyledText;
use super::view::{BufferView, BufferViewCreateParams};
use super::{BufferViewID, CursorStyle};

fn default_hl_for_line(line: RopeSlice, color: Color) -> StyledText {
    let lch = rope_trim_newlines(line).len_chars();
    StyledText::new(lch, TextStyle::default(), color, None)
}

pub(crate) struct Buffer {
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
    styled_lines: Vec<StyledText>,
    tab_width: usize,
    parser: Option<Parser>,
    hl_query: Option<Rc<Query>>,
    tree: Option<Tree>,
    theme: Rc<Theme>,
}

impl Buffer {
    // -------- View management ----------------
    pub(crate) fn new_view(&mut self, id: &BufferViewID, params: BufferViewCreateParams) {
        self.views.insert(
            id.clone(),
            BufferView::new(
                params,
                self.theme.clone(),
                &self.data,
                &self.styled_lines,
                self.tab_width,
            ),
        );
    }

    pub(crate) fn set_view_rect(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views
            .get_mut(id)
            .unwrap()
            .set_rect(rect, &self.data, &self.styled_lines);
    }

    pub(crate) fn draw_view(&mut self, id: &BufferViewID, painter: &mut Painter) {
        self.views.get_mut(id).unwrap().draw(painter);
    }

    pub(crate) fn check_view_needs_redraw(&mut self, id: &BufferViewID) -> bool {
        self.views.get(id).unwrap().needs_redraw
    }

    pub(crate) fn remove_view(&mut self, id: &BufferViewID) {
        self.views.remove(id);
    }

    pub(crate) fn scroll_view(&mut self, id: &BufferViewID, vec: Vector2D<i32, PixelSize>) {
        self.views
            .get_mut(id)
            .unwrap()
            .scroll(vec, &self.data, &self.styled_lines);
    }

    // -------- View cursor manipulation ----------------
    pub(crate) fn move_view_cursor(&mut self, id: &BufferViewID, mov: Motion) {
        let view = self.views.get_mut(id).unwrap();
        match mov {
            Motion::Up(n) => {
                if view.cursor.line_num == 0 {
                    view.cursor.char_idx = 0;
                    view.cursor.line_cidx = 0;
                    view.cursor.line_gidx = 0;
                    view.cursor.line_global_x = 0;
                } else {
                    if view.cursor.line_num < n {
                        view.cursor.line_num = 0;
                    } else {
                        view.cursor.line_num -= n;
                    }
                }
                view.cursor.sync_global_x(&self.data, self.tab_width);
            }
            Motion::Down(n) => {
                view.cursor.line_num += n;
                if view.cursor.line_num >= self.data.len_lines() {
                    view.cursor.char_idx = self.data.len_chars();
                    view.cursor
                        .sync_and_update_char_idx_left(&self.data, self.tab_width);
                } else {
                    view.cursor.sync_global_x(&self.data, self.tab_width);
                }
            }
            Motion::Left(n) => {
                if view.cursor.line_cidx <= n {
                    view.cursor.line_cidx = 0;
                } else {
                    view.cursor.line_cidx -= n;
                }
                view.cursor
                    .sync_line_cidx_gidx_left(&self.data, self.tab_width);
            }
            Motion::Right(n) => {
                view.cursor.line_cidx += n;
                view.cursor
                    .sync_line_cidx_gidx_right(&self.data, self.tab_width);
            }
            Motion::LineStart => {
                view.cursor.line_cidx = 0;
                view.cursor
                    .sync_line_cidx_gidx_right(&self.data, self.tab_width);
            }
            Motion::LineEnd => {
                let lc = rope_trim_newlines(self.data.line(view.cursor.line_num)).len_chars();
                view.cursor.line_cidx = lc;
                view.cursor
                    .sync_line_cidx_gidx_right(&self.data, self.tab_width);
            }
            Motion::ToLine(linum) => {
                view.cursor.line_num = min(linum, self.data.len_lines() - 1);
                view.cursor.line_cidx = 0;
                view.cursor
                    .sync_line_cidx_gidx_right(&self.data, self.tab_width);
            }
        }
        view.snap_to_cursor(&self.data, &self.styled_lines);
    }

    pub(crate) fn move_view_cursor_to_point(
        &mut self,
        id: &BufferViewID,
        point: Point2D<u32, PixelSize>,
    ) {
        let view = self.views.get_mut(id).unwrap();
        view.move_cursor_to_point(point, &self.data, &self.styled_lines, self.tab_width);
    }

    pub(crate) fn set_view_cursor_style(&mut self, id: &BufferViewID, style: CursorStyle) {
        let view = self.views.get_mut(id).unwrap();
        view.cursor.style = style;
        view.cursor
            .sync_line_cidx_gidx_left(&self.data, self.tab_width);
        view.snap_to_cursor(&self.data, &self.styled_lines);
    }

    // -------- View edits -----------------
    pub(crate) fn view_insert_char(&mut self, id: &BufferViewID, c: char) {
        let view = self.views.get_mut(id).unwrap();
        let cidx = view.cursor.char_idx;
        let lc = self.data.len_chars();
        let linum = view.cursor.line_num;
        let mut end_linum = linum;
        let mut end_cidx = cidx + 1;

        match c {
            // Insert pair
            '[' | '{' | '(' => {
                self.data.insert_char(cidx, c);
                if cidx + 1 == lc || self.data.char(cidx + 1).is_whitespace() {
                    match c {
                        '[' => self.data.insert_char(cidx + 1, ']'),
                        '{' => self.data.insert_char(cidx + 1, '}'),
                        '(' => self.data.insert_char(cidx + 1, ')'),
                        _ => unreachable!(),
                    }
                    end_cidx += 1;
                }
            }
            // Maybe insert pair, maybe skip
            '"' | '\'' => {
                if cidx == lc || self.data.char(cidx) != c {
                    self.data.insert_char(cidx, c);
                    self.data.insert_char(cidx + 1, c);
                    end_cidx += 1;
                } else {
                    return self.move_view_cursor(id, Motion::Right(1));
                }
            }
            // Maybe skip insert
            ']' | '}' | ')' => {
                if cidx == lc || self.data.char(cidx) != c {
                    self.data.insert_char(cidx, c);
                } else {
                    return self.move_view_cursor(id, Motion::Right(1));
                }
            }
            // Maybe insert twice?
            '\n' | ' ' => {
                self.data.insert_char(cidx, c);
                if c == '\n' {
                    end_linum += 1;
                }
                if cidx > 0 && cidx + 1 < lc {
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
                        end_cidx += 1;
                    }
                }
            }
            c => self.data.insert_char(cidx, c),
        }

        let fgcol = self.theme.textview.foreground;
        self.styled_lines[linum] = default_hl_for_line(self.data.line(linum), fgcol);
        for i in linum..end_linum {
            self.styled_lines
                .insert(i + 1, default_hl_for_line(self.data.line(i + 1), fgcol));
        }

        self.edit_tree(self.data.clone(), cidx, cidx, end_cidx);
        let (end_byte, end_col) = {
            let llen = self.data.line(end_linum).len_bytes();
            let lb = self.data.line_to_byte(end_linum);
            (lb + llen, llen)
        };
        self.rehighlight_range(tree_sitter::Range {
            start_byte: self.data.line_to_byte(linum),
            end_byte: end_byte,
            start_point: Point::new(linum, 0),
            end_point: Point::new(end_linum, end_col),
        });

        for view in self.views.values_mut() {
            if view.cursor.char_idx >= cidx {
                view.cursor.char_idx += 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape(&self.data, &self.styled_lines);
            view.snap_to_cursor(&self.data, &self.styled_lines);
        }
    }

    pub(crate) fn view_delete(&mut self, id: &BufferViewID, mov: Motion) {
        let view = self.views.get_mut(id).unwrap();
        let cidx = view.cursor.char_idx;
        let lc = self.data.len_chars();
        let mut linum = view.cursor.line_num;
        let mut move_to_start = false;
        let (start_cidx, end_cidx) = match mov {
            Motion::Left(n) => {
                if cidx == 0 {
                    return;
                }
                let start_cidx = if cidx <= n { 0 } else { cidx - n };
                let start_line = self.data.char_to_line(start_cidx);
                for _ in start_line..linum {
                    self.styled_lines.remove(start_line + 1);
                }
                linum = start_line;
                (start_cidx, cidx)
            }
            Motion::Right(n) => {
                if cidx == self.data.len_chars() {
                    return;
                }
                let end_cidx = if cidx + n >= lc { lc } else { cidx + n };
                let end_line = self.data.char_to_line(end_cidx);
                for _ in linum..end_line {
                    self.styled_lines.remove(linum + 1);
                }
                (cidx, end_cidx)
            }
            Motion::Down(mut n) => {
                let start_linum = view.cursor.line_num;
                let (start_cidx, end_cidx) = if start_linum + n >= self.data.len_lines() {
                    linum -= 1;
                    (
                        self.data.line_to_char(linum)
                            + rope_trim_newlines(self.data.line(linum)).len_chars(),
                        self.data.len_chars(),
                    )
                } else {
                    (
                        cidx - view.cursor.line_cidx,
                        self.data.line_to_char(start_linum + n),
                    )
                };
                if start_linum + n > self.data.len_lines() {
                    n = self.data.len_lines() - start_linum;
                }
                for _ in 0..n {
                    self.styled_lines.remove(start_linum);
                }
                move_to_start = true;
                (start_cidx, end_cidx)
            }
            _ => unimplemented!(),
        };

        let old_rope = self.data.clone();

        self.data.remove(start_cidx..end_cidx);
        self.styled_lines[linum] =
            default_hl_for_line(self.data.line(linum), self.theme.textview.foreground);

        self.edit_tree(old_rope, start_cidx, end_cidx, start_cidx);

        let (start_byte, end_byte) = {
            let llen = self.data.line(linum).len_bytes();
            let lb = self.data.line_to_byte(linum);
            (lb, lb + llen)
        };
        self.rehighlight_range(tree_sitter::Range {
            start_byte: start_byte,
            end_byte: end_byte,
            start_point: Point::new(linum, 0),
            end_point: Point::new(linum, end_byte - start_byte),
        });

        for (vid, view) in self.views.iter_mut() {
            if view.cursor.char_idx >= end_cidx {
                view.cursor.char_idx -= end_cidx - start_cidx;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            } else if view.cursor.char_idx >= start_cidx {
                if move_to_start && vid == id {
                    view.cursor.line_num = linum;
                    view.cursor.line_cidx = 0;
                    view.cursor
                        .sync_line_cidx_gidx_left(&self.data, self.tab_width);
                } else {
                    view.cursor.char_idx = start_cidx;
                    view.cursor
                        .sync_and_update_char_idx_left(&self.data, self.tab_width);
                }
            }
            view.reshape(&self.data, &self.styled_lines);
            view.snap_to_cursor(&self.data, &self.styled_lines);
        }
    }

    // -------- Create buffer ----------------
    pub(super) fn empty(theme: Rc<Theme>) -> Buffer {
        let styled = StyledText::new(0, TextStyle::default(), theme.textview.foreground, None);
        Buffer {
            tab_width: 8,
            data: Rope::new(),
            views: FnvHashMap::default(),
            styled_lines: vec![styled],
            parser: None,
            hl_query: None,
            tree: None,
            theme,
        }
    }

    pub(super) fn from_file(path: &str, ts_core: &TsCore, theme: Rc<Theme>) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| {
                let mut styled_lines = Vec::new();
                for line in rope.lines() {
                    styled_lines.push(default_hl_for_line(line, theme.textview.foreground));
                }
                let (parser, hl_query) = Path::new(path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .and_then(|s| ts_core.parser_from_extension(s))
                    .map(|(p, q)| (Some(p), Some(q)))
                    .unwrap_or((None, None));
                let mut ret = Buffer {
                    tab_width: 8,
                    data: rope,
                    views: FnvHashMap::default(),
                    styled_lines,
                    parser,
                    hl_query,
                    tree: None,
                    theme,
                };
                ret.recreate_parse_tree();
                ret
            })
    }

    pub(super) fn reload_from_file(&mut self, path: &str, ts_core: &TsCore) -> IOResult<()> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| {
                self.data = rope;
                self.styled_lines.clear();
                for line in self.data.lines() {
                    self.styled_lines
                        .push(default_hl_for_line(line, self.theme.textview.foreground));
                }
                let (parser, hl_query) = Path::new(path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .and_then(|s| ts_core.parser_from_extension(s))
                    .map(|(p, q)| (Some(p), Some(q)))
                    .unwrap_or((None, None));
                self.parser = parser;
                self.hl_query = hl_query;
                self.recreate_parse_tree();
            })
    }

    // -------- Parsing stuff ----------------
    fn recreate_parse_tree(&mut self) {
        let rope = self.data.clone();
        if let Some(parser) = &mut self.parser {
            let t = parser
                .parse_with(
                    &mut |boff, _| {
                        if boff >= rope.len_bytes() {
                            ""
                        } else {
                            let (ch, cb, _, _) = rope.chunk_at_byte(boff);
                            &ch[boff - cb..]
                        }
                    },
                    None,
                )
                .expect("failed to parse");
            self.tree = Some(t.clone());
            self.rehighlight_range(tree_sitter::Range {
                start_byte: 0,
                end_byte: self.data.len_bytes(),
                start_point: Point::new(0, 0),
                end_point: Point::new(
                    self.data.len_lines(),
                    self.data.len_bytes() - self.data.line_to_byte(self.data.len_lines()),
                ),
            })
        }
    }

    fn edit_tree(
        &mut self,
        rope: Rope,
        start_cidx: usize,
        old_end_cidx: usize,
        new_end_cidx: usize,
    ) {
        if self.tree.is_none() {
            return;
        }
        let start_bidx = rope.char_to_byte(start_cidx);
        let old_end_bidx = rope.char_to_byte(old_end_cidx);
        let new_end_bidx = rope.char_to_byte(new_end_cidx);
        let start_linum = rope.byte_to_line(start_bidx);
        let start_linoff = start_bidx - rope.line_to_byte(start_linum);
        let old_end_linum = rope.byte_to_line(old_end_bidx);
        let old_end_linoff = old_end_bidx - rope.line_to_byte(old_end_linum);
        let new_end_linum = rope.byte_to_line(new_end_bidx);
        let new_end_linoff = new_end_bidx - rope.line_to_byte(new_end_linum);

        let mut tree = self.tree.take().unwrap();
        tree.edit(&InputEdit {
            start_byte: start_bidx,
            old_end_byte: old_end_bidx,
            new_end_byte: new_end_bidx,
            start_position: Point::new(start_linum, start_linoff),
            old_end_position: Point::new(old_end_linum, old_end_linoff),
            new_end_position: Point::new(new_end_linum, new_end_linoff),
        });

        let rope = self.data.clone();
        if let Some(parser) = &mut self.parser {
            let t = parser
                .parse_with(
                    &mut |boff, _| {
                        if boff >= rope.len_bytes() {
                            ""
                        } else {
                            let (ch, cb, _, _) = rope.chunk_at_byte(boff);
                            &ch[boff - cb..]
                        }
                    },
                    Some(&mut tree),
                )
                .expect("failed to parse");
            self.tree = Some(t.clone());
            for range in t.changed_ranges(&tree) {
                self.rehighlight_range(range);
            }
        }
    }

    fn rehighlight_range(&mut self, mut range: tree_sitter::Range) {
        self.expand_rehighlight_range(&mut range);

        let mut linum = range.start_point.row;
        for line in self.data.lines_at(range.start_point.row) {
            self.styled_lines[linum] = default_hl_for_line(line, self.theme.textview.foreground);
            linum += 1;
            if linum >= range.end_point.row {
                break;
            }
        }

        if let Some(t) = &self.tree {
            if let Some(hl_query) = &self.hl_query {
                let rope = self.data.clone();
                let mut cursor = QueryCursor::new();
                cursor.set_byte_range(range.start_byte, range.end_byte);
                let mut last_pos = Point::new(0, 0);
                let mut buf = String::new();
                for (query_match, _) in cursor.captures(hl_query, t.root_node(), |node| {
                    let range = node.byte_range();
                    let range = rope.byte_to_char(range.start)..rope.byte_to_char(range.end);
                    // FIXME: Optimize this
                    format!("{}", rope.slice(range))
                }) {
                    for capture in query_match.captures {
                        let node = capture.node;
                        let idx = capture.index;
                        let mut start = node.start_position();
                        let end = node.end_position();
                        if end.row < last_pos.row
                            || (end.row == last_pos.row && end.column <= last_pos.column)
                        {
                            continue;
                        } else if start.row < last_pos.row
                            || (start.row == last_pos.row && start.column < last_pos.column)
                        {
                            start = last_pos;
                        }
                        let mut elem = None;
                        buf.clear();
                        let capture_name = &hl_query.capture_names()[idx as usize];
                        for split in capture_name.split('.') {
                            if buf.len() > 0 {
                                buf.push('.');
                            }
                            buf.push_str(split);
                            if let Some(se) = self.theme.syntax.get(&buf) {
                                elem = Some(se);
                            }
                        }
                        if let Some(elem) = elem {
                            let style = TextStyle::new(elem.weight, elem.slant);
                            let fg = elem.foreground;
                            let sl = rope_trim_newlines(self.data.line(start.row));
                            let slc = sl.byte_to_char(start.column);
                            let elc = self.data.line(end.row).byte_to_char(end.column);
                            if start.row == end.row {
                                if elc > slc {
                                    self.styled_lines[start.row].set(slc..elc, style, fg, None);
                                }
                            } else {
                                self.styled_lines[start.row].set(
                                    slc..sl.len_chars(),
                                    style,
                                    fg,
                                    None,
                                );
                                self.styled_lines[end.row].set(0..elc, style, fg, None);
                                let mut linum = start.row + 1;
                                for line in self.data.lines_at(linum) {
                                    if linum >= end.row {
                                        break;
                                    }
                                    let lc = rope_trim_newlines(line).len_chars();
                                    self.styled_lines[linum] = StyledText::new(lc, style, fg, None);
                                    linum += 1;
                                }
                            }
                            last_pos = end;
                        }
                    }
                }
            }
        }
    }

    fn expand_rehighlight_range(&self, range: &mut tree_sitter::Range) {
        range.start_point.column = 0;
        range.start_byte = self.data.line_to_byte(range.start_point.row);
        if range.end_point.row == self.data.len_lines() {
            return;
        }
        let end_line = self.data.line(range.end_point.row);
        range.end_point.column = end_line.len_bytes();
        range.end_byte = self.data.line_to_byte(range.end_point.row) + range.end_point.column;
    }
}
