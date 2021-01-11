// (C) 2021 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::text::CursorStyle;

use super::{Buffer, BufferViewId};

impl Buffer {
    // -------- Cursor movement --------
    pub(crate) fn move_view_cursor_up(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_up(&self.view(view_id).cursor, n);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_down(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_down(&self.view(view_id).cursor, n);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_left(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_left(&self.view(view_id).cursor, n);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_right(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_right(&self.view(view_id).cursor, n);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_to_line_start(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_line_start(&self.view(view_id).cursor, n);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_to_line_end(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_line_end(&self.view(view_id).cursor, n);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_to_first_non_blank(&mut self, view_id: &BufferViewId) {
        self.view_mut(view_id).cursor = self.cursor_first_non_blank(&self.view(view_id).cursor);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_to_line(&mut self, view_id: &BufferViewId, linum: usize) {
        self.view_mut(view_id).cursor = self.cursor_line(&self.view(view_id).cursor, linum);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_to_last_line(&mut self, view_id: &BufferViewId) {
        let len_lines = self.shared.borrow().rope.len_lines();
        self.move_view_cursor_to_line(view_id, len_lines - 1);
    }

    pub(crate) fn move_view_cursor_to_view_first_line(&mut self, view_id: &BufferViewId) {
        self.view_mut(view_id).move_cursor_to_first_line();
        self.move_view_cursor_to_first_non_blank(view_id);
    }

    pub(crate) fn move_view_cursor_to_view_middle_line(&mut self, view_id: &BufferViewId) {
        self.view_mut(view_id).move_cursor_to_middle_line();
        self.move_view_cursor_to_first_non_blank(view_id);
    }

    pub(crate) fn move_view_cursor_to_view_last_line(&mut self, view_id: &BufferViewId) {
        self.view_mut(view_id).move_cursor_to_last_line();
        self.move_view_cursor_to_first_non_blank(view_id);
    }

    pub(crate) fn move_view_cursor_word(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_word(&self.view(view_id).cursor, n, false);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_word_extended(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_word(&self.view(view_id).cursor, n, true);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_word_end(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_word_end(&self.view(view_id).cursor, n, false);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_word_end_extended(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_word_end(&self.view(view_id).cursor, n, true);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_back(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_back(&self.view(view_id).cursor, n, false);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_back_extended(&mut self, view_id: &BufferViewId, n: usize) {
        self.view_mut(view_id).cursor = self.cursor_back(&self.view(view_id).cursor, n, true);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn set_view_cursor_style(&mut self, view_id: &BufferViewId, style: CursorStyle) {
        let shared = self.shared.borrow();
        let view = self.views.get_mut(view_id).unwrap();
        view.cursor.style = style;
        view.cursor
            .sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn snap_to_cursor(&mut self, view_id: &BufferViewId, update_global_x: bool) {
        let view = self.views.get_mut(view_id).unwrap();
        view.snap_to_cursor(update_global_x);
    }
}
