// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fs::File;
use std::io::Result as IOResult;
use std::path::Path;
use std::rc::Rc;

use euclid::{Point2D, Rect, Vector2D};
use fnv::FnvHashMap;
use ropey::Rope;
use tree_sitter::{InputEdit, Parser, Point, Query, QueryCursor, Tree};

use crate::common::{rope_trim_newlines, PixelSize};
use crate::painter::WidgetPainter;
use crate::style::{Color, TextStyle};
use crate::ts::TsCore;

use super::view::{BufferView, BufferViewCreateParams, StyledText};
use super::{BufferID, BufferViewID};

pub(crate) struct Buffer {
    buf_id: BufferID,
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
    styled_lines: Vec<StyledText>,
    tab_width: usize,
    parser: Option<Parser>,
    hl_query: Option<Rc<Query>>,
    tree: Option<Tree>,
}

impl Buffer {
    // -------- View management ----------------
    pub(crate) fn new_view(&mut self, id: &BufferViewID, params: BufferViewCreateParams) {
        self.views.insert(
            id.clone(),
            BufferView::new(params, &self.data, &self.styled_lines, self.tab_width),
        );
    }

    pub(crate) fn set_view_rect(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views
            .get_mut(id)
            .unwrap()
            .set_rect(rect, &self.data, &self.styled_lines);
    }

    pub(crate) fn draw_view(&mut self, id: &BufferViewID, painter: &mut WidgetPainter) {
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

    // -------- View cursor motion ----------------
    pub(crate) fn move_view_cursor_up(&mut self, id: &BufferViewID, n: usize) {
        let view = self.views.get_mut(id).unwrap();
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
        view.snap_to_cursor(&self.data, &self.styled_lines);
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
        view.snap_to_cursor(&self.data, &self.styled_lines);
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
        view.snap_to_cursor(&self.data, &self.styled_lines);
    }

    pub(crate) fn move_view_cursor_right(&mut self, id: &BufferViewID, n: usize) {
        let view = self.views.get_mut(id).unwrap();
        view.cursor.line_cidx += n;
        view.cursor
            .sync_line_cidx_gidx_right(&self.data, self.tab_width);
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

    // -------- View edits -----------------
    pub(crate) fn view_insert_char(&mut self, id: &BufferViewID, c: char) {
        let view = self.views.get_mut(id).unwrap();
        let cidx = view.cursor.char_idx;
        let linum = view.cursor.line_num;
        let mut end_linum = linum;
        let mut end_cidx = cidx + 1;
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
                    end_cidx += 1;
                }
            }
            // Maybe insert pair, maybe skip
            '"' | '\'' => {
                if self.data.char(cidx) != c {
                    self.data.insert_char(cidx, c);
                    self.data.insert_char(cidx + 1, c);
                    end_cidx += 1;
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
                        end_cidx += 1;
                    }
                }
            }
            c => self.data.insert_char(cidx, c),
        }
        self.edit_tree(cidx, cidx, end_cidx);
        let lch = rope_trim_newlines(self.data.line(linum)).len_chars();
        let mut styled = StyledText::new();
        styled.push(lch, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
        self.styled_lines[linum] = styled;
        for i in linum..end_linum {
            let lch = rope_trim_newlines(self.data.line(i + 1)).len_chars();
            let mut styled = StyledText::new();
            styled.push(lch, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
            self.styled_lines.insert(i + 1, styled);
        }
        for view in self.views.values_mut() {
            if view.cursor.char_idx >= cidx {
                view.cursor.char_idx += 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, &self.styled_lines, linum);
            for i in linum..end_linum {
                view.insert_line(&self.data, &self.styled_lines, i + 1);
            }
            view.snap_to_cursor(&self.data, &self.styled_lines);
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
            self.styled_lines.remove(linum);
            linum -= 1;
        }
        self.edit_tree(cidx - 1, cidx, cidx - 1);
        let lch = rope_trim_newlines(self.data.line(linum)).len_chars();
        let mut styled = StyledText::new();
        styled.push(lch, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
        self.styled_lines[linum] = styled;
        for view in self.views.values_mut() {
            if view.cursor.char_idx >= cidx {
                view.cursor.char_idx -= 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, &self.styled_lines, linum);
            if is_beg {
                view.delete_line(&self.data, &self.styled_lines, linum + 1);
            }
            view.snap_to_cursor(&self.data, &self.styled_lines);
        }
    }

    pub(crate) fn view_delete_right(&mut self, id: &BufferViewID) {
        let view = self.views.get_mut(id).unwrap();
        if view.cursor.char_idx == self.data.len_chars() {
            return;
        }
        let cidx = view.cursor.char_idx;
        let linum = view.cursor.line_num;
        let len_lines = self.data.len_lines();
        self.data.remove(cidx..cidx + 1);
        let del_end = self.data.len_lines() < len_lines;
        if del_end {
            self.styled_lines.remove(linum + 1);
        }
        self.edit_tree(cidx, cidx + 1, cidx);
        let lch = rope_trim_newlines(self.data.line(linum)).len_chars();
        let mut styled = StyledText::new();
        styled.push(lch, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
        self.styled_lines[linum] = styled;
        for view in self.views.values_mut() {
            if view.cursor.char_idx > cidx {
                view.cursor.char_idx -= 1;
            }
            if view.cursor.char_idx >= cidx {
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, &self.styled_lines, linum);
            if del_end {
                view.delete_line(&self.data, &self.styled_lines, linum + 1);
            }
            view.snap_to_cursor(&self.data, &self.styled_lines);
        }
    }

    // -------- Create buffer ----------------
    pub(super) fn empty(buf_id: BufferID) -> Buffer {
        let mut styled = StyledText::new();
        styled.push(0, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
        Buffer {
            buf_id: buf_id,
            tab_width: 8,
            data: Rope::new(),
            views: FnvHashMap::default(),
            styled_lines: vec![styled],
            parser: None,
            hl_query: None,
            tree: None,
        }
    }

    pub(super) fn from_file(buf_id: BufferID, path: &str, ts_core: &TsCore) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| {
                let mut styled_lines = Vec::new();
                for line in rope.lines() {
                    let lch = rope_trim_newlines(line).len_chars();
                    let mut styled = StyledText::new();
                    styled.push(lch, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
                    styled_lines.push(styled);
                }
                let (parser, hl_query) = Path::new(path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .and_then(|s| ts_core.parser_from_extension(s))
                    .map(|(p, q)| (Some(p), Some(q)))
                    .unwrap_or((None, None));
                let mut ret = Buffer {
                    buf_id: buf_id,
                    tab_width: 8,
                    data: rope,
                    views: FnvHashMap::default(),
                    styled_lines,
                    parser,
                    hl_query,
                    tree: None,
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

            if let Some(hl_query) = &self.hl_query {
                let mut cursor = QueryCursor::new();
                for (query_match, i) in cursor.captures(hl_query, t.root_node(), |node| {
                    let range = node.byte_range();
                    let range = rope.byte_to_char(range.start)..rope.byte_to_char(range.end);
                    format!("{}", rope.slice(range))
                }) {
                    for capture in query_match.captures {
                        let node = capture.node;
                        let idx = capture.index;
                        let brange = node.byte_range();
                        let crange = rope.byte_to_char(brange.start)..rope.byte_to_char(brange.end);
                        /*
                        println!(
                            "{:?} -> {} -> {}",
                            brange,
                            rope.slice(crange),
                            hl_query.capture_names()[idx as usize]
                        );
                        */
                    }
                }
            }

            self.tree = Some(t);
        }
    }

    fn edit_tree(&mut self, start_cidx: usize, old_end_cidx: usize, new_end_cidx: usize) {
        if self.tree.is_none() {
            return;
        }
        let start_bidx = self.data.char_to_byte(start_cidx);
        let old_end_bidx = self.data.char_to_byte(old_end_cidx);
        let new_end_bidx = self.data.char_to_byte(new_end_cidx);
        let start_linum = self.data.byte_to_line(start_bidx);
        let start_linoff = start_bidx - self.data.line_to_byte(start_linum);
        let old_end_linum = self.data.byte_to_line(old_end_bidx);
        let old_end_linoff = old_end_bidx - self.data.line_to_byte(old_end_linum);
        let new_end_linum = self.data.byte_to_line(new_end_bidx);
        let new_end_linoff = new_end_bidx - self.data.line_to_byte(new_end_linum);

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
            self.tree = Some(t);
        }
    }

    fn rehighlight_from(&mut self, start_linum: usize, sync_upto: usize) {
        // TODO
    }
}
