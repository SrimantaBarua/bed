// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::File;
use std::io::Result as IOResult;
use std::rc::Rc;

use euclid::{Point2D, Rect, Vector2D};
use fnv::FnvHashMap;
use ropey::Rope;

use crate::common::PixelSize;
use crate::painter::Painter;

use super::view::{CursorStyle, View};
use super::{BufferBedHandle, BufferViewId};

#[derive(Clone)]
pub(crate) struct BufferHandle(Rc<RefCell<Buffer>>);

impl PartialEq for BufferHandle {
    fn eq(&self, other: &BufferHandle) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for BufferHandle {}

impl BufferHandle {
    // -------- View manipulation --------
    pub(crate) fn new_view(&mut self, view_id: &BufferViewId, rect: Rect<u32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        let view = View::new(inner.bed_handle.clone(), rect);
        inner.views.insert(view_id.clone(), view);
    }

    pub(crate) fn set_view_rect(&mut self, view_id: &BufferViewId, rect: Rect<u32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.set_rect(rect, &inner.rope, inner.tab_width);
    }

    pub(crate) fn draw_view(&mut self, view_id: &BufferViewId, painter: &mut Painter) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.draw(&inner.rope, painter, inner.tab_width);
    }

    pub(crate) fn scroll_view(&mut self, view_id: &BufferViewId, scroll: Vector2D<i32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.scroll(scroll, &inner.rope, inner.tab_width);
    }

    // -------- Cursor movement --------
    pub(crate) fn move_view_cursor_up(&mut self, view_id: &BufferViewId, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        let cursor = &mut view.cursor;
        if cursor.line_num == 0 {
            cursor.reset();
        } else {
            if n < cursor.line_num {
                cursor.line_num -= n;
            } else {
                cursor.line_num = 0;
            }
            cursor.sync_global_x(&inner.rope, inner.tab_width);
        }
        view.snap_to_cursor(&inner.rope, inner.tab_width);
        inner.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_down(&mut self, view_id: &BufferViewId, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        let cursor = &mut view.cursor;
        cursor.line_num += n;
        if cursor.line_num >= inner.rope.len_lines() {
            cursor.cidx = inner.rope.len_chars();
            cursor.sync_and_update_char_idx_left(&inner.rope, inner.tab_width);
        } else {
            cursor.sync_global_x(&inner.rope, inner.tab_width);
        }
        view.snap_to_cursor(&inner.rope, inner.tab_width);
        inner.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_left(&mut self, view_id: &BufferViewId, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        let cursor = &mut view.cursor;
        if cursor.line_cidx < n {
            cursor.line_cidx = 0;
        } else {
            cursor.line_cidx -= n;
        }
        cursor.sync_line_cidx_gidx_left(&inner.rope, inner.tab_width);
        view.snap_to_cursor(&inner.rope, inner.tab_width);
        inner.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_right(&mut self, view_id: &BufferViewId, n: usize) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        let cursor = &mut view.cursor;
        cursor.line_cidx += n;
        cursor.sync_line_cidx_gidx_right(&inner.rope, inner.tab_width);
        view.snap_to_cursor(&inner.rope, inner.tab_width);
        inner.bed_handle.request_redraw();
    }

    pub(crate) fn set_view_cursor_style(&mut self, view_id: &BufferViewId, style: CursorStyle) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.cursor.style = style;
        view.cursor
            .sync_line_cidx_gidx_left(&inner.rope, inner.tab_width);
        view.snap_to_cursor(&inner.rope, inner.tab_width);
        inner.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_to_point(
        &mut self,
        view_id: &BufferViewId,
        point: Point2D<i32, PixelSize>,
    ) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.move_cursor_to_point(point, &inner.rope, inner.tab_width);
    }

    // -------- Editing --------
    pub(crate) fn insert_char(&mut self, view_id: &BufferViewId, c: char) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        let cidx = view.cursor.cidx;
        inner.rope.insert_char(cidx, c);
        for view in inner.views.values_mut() {
            if view.cursor.cidx >= cidx {
                view.cursor.cidx += 1;
                view.cursor
                    .sync_and_update_char_idx_left(&inner.rope, inner.tab_width);
            }
        }
        let view = inner.views.get_mut(view_id).unwrap();
        view.snap_to_cursor(&inner.rope, inner.tab_width);
        inner.bed_handle.request_redraw();
    }

    pub(crate) fn delete_left(&mut self, view_id: &BufferViewId, mut n: usize) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        let cidx = view.cursor.cidx;
        if cidx < n {
            n = cidx;
        }
        let start_cidx = cidx - n;
        inner.rope.remove(start_cidx..cidx);
        for view in inner.views.values_mut() {
            if view.cursor.cidx >= cidx {
                view.cursor.cidx -= n;
            } else if view.cursor.cidx > start_cidx {
                view.cursor.cidx = start_cidx;
            }
            if view.cursor.cidx >= start_cidx {
                view.cursor
                    .sync_and_update_char_idx_left(&inner.rope, inner.tab_width);
            }
        }
        let view = inner.views.get_mut(view_id).unwrap();
        view.snap_to_cursor(&inner.rope, inner.tab_width);
        inner.bed_handle.request_redraw();
    }

    pub(crate) fn delete_right(&mut self, view_id: &BufferViewId, mut n: usize) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        let mut cidx = view.cursor.cidx;
        let pre_len_chars = inner.rope.len_chars();
        assert!(cidx <= pre_len_chars);
        if cidx == pre_len_chars {
            cidx = pre_len_chars - 1;
            n = 1;
        } else if cidx + n >= pre_len_chars {
            n = pre_len_chars - cidx;
        }
        let end_cidx = cidx + n;
        inner.rope.remove(cidx..end_cidx);
        for view in inner.views.values_mut() {
            if view.cursor.cidx >= end_cidx {
                view.cursor.cidx -= n;
            } else if view.cursor.cidx > cidx {
                view.cursor.cidx = cidx;
            }
            if view.cursor.cidx >= cidx {
                view.cursor
                    .sync_and_update_char_idx_left(&inner.rope, inner.tab_width);
            }
        }
        let view = inner.views.get_mut(view_id).unwrap();
        view.snap_to_cursor(&inner.rope, inner.tab_width);
        inner.bed_handle.request_redraw();
    }

    // -------- Buffer creation --------
    pub(super) fn create_empty(bed_handle: BufferBedHandle) -> BufferHandle {
        BufferHandle(Rc::new(RefCell::new(Buffer::empty(bed_handle))))
    }

    pub(super) fn create_from_file(
        path: &str,
        bed_handle: BufferBedHandle,
    ) -> IOResult<BufferHandle> {
        Buffer::from_file(path, bed_handle).map(|buf| BufferHandle(Rc::new(RefCell::new(buf))))
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        let inner = &mut *self.0.borrow_mut();
        inner.reload_from_file(path)
    }
}

struct Buffer {
    views: FnvHashMap<BufferViewId, View>,
    bed_handle: BufferBedHandle,
    rope: Rope,
    tab_width: usize,
}

impl Buffer {
    fn empty(bed_handle: BufferBedHandle) -> Buffer {
        Buffer {
            views: FnvHashMap::default(),
            rope: Rope::new(),
            bed_handle,
            tab_width: 8,
        }
    }

    fn from_file(path: &str, bed_handle: BufferBedHandle) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| Buffer {
                rope,
                bed_handle,
                views: FnvHashMap::default(),
                tab_width: 8,
            })
    }

    fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| {
                self.rope = rope;
                for view in self.views.values_mut() {
                    view.scroll_to_top();
                }
            })
    }
}
