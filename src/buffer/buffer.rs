// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fs::File;
use std::io::Result as IOResult;

use euclid::{Point2D, Rect, Vector2D};
use fnv::FnvHashMap;
use ropey::Rope;

use crate::common::PixelSize;
use crate::painter::WidgetPainter;

use super::view::{BufferView, BufferViewCreateParams};
use super::BufferViewID;

pub(crate) struct Buffer {
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
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
        self.data.insert_char(view.cursor.char_idx, c);
        let cidx = view.cursor.char_idx;
        let linum = view.cursor.line_num;
        let is_newline = c == '\n';
        for view in self.views.values_mut() {
            if view.cursor.char_idx >= cidx {
                view.cursor.char_idx += 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
            }
            view.reshape_line(&self.data, linum);
            if is_newline {
                view.insert_line(&self.data, linum + 1);
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
    pub(super) fn empty() -> Buffer {
        Buffer {
            tab_width: 8,
            data: Rope::new(),
            views: FnvHashMap::default(),
        }
    }

    pub(super) fn from_file(path: &str) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| Buffer {
                tab_width: 8,
                data: rope,
                views: FnvHashMap::default(),
            })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| self.data = rope)
    }
}
