// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::{RefCell, RefMut};
use std::cmp::min;
use std::fs::File;
use std::io::{Result as IOResult, Write as IOWrite};
use std::ops::Range;
use std::rc::Rc;

use euclid::{Point2D, Rect, Vector2D};
use fnv::FnvHashMap;
use ropey::Rope;
use tree_sitter::{InputEdit, Point as TsPoint, QueryCursor, Tree};

use crate::common::{rope_trim_newlines, AbsPath, PixelSize};
use crate::language::Language;
use crate::painter::Painter;
use crate::style::{StyleRanges, TextStyle};
use crate::text::CursorStyle;
use crate::ts::{TsCore, TsLang};

use super::rope_stuff::{space_containing, word_containing};
use super::view::{View, ViewCursor};
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
    pub(crate) fn buffer(&mut self) -> RefMut<Buffer> {
        self.0.borrow_mut()
    }

    pub(crate) fn new_view(&mut self, view_id: &BufferViewId, rect: Rect<u32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        let view = View::new(inner.bed_handle.clone(), rect);
        inner.views.insert(view_id.clone(), view);
    }

    pub(crate) fn set_view_rect(&mut self, view_id: &BufferViewId, rect: Rect<u32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.set_rect(rect, &inner.rope, inner.tab_width, &inner.styles);
    }

    pub(crate) fn draw_view(&mut self, view_id: &BufferViewId, painter: &mut Painter) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.draw(painter, &inner.rope, inner.tab_width, &inner.styles);
    }

    pub(crate) fn scroll_view(&mut self, view_id: &BufferViewId, scroll: Vector2D<i32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.scroll(scroll, &inner.rope, inner.tab_width, &inner.styles);
    }

    pub(crate) fn move_view_cursor_to_point(
        &mut self,
        view_id: &BufferViewId,
        point: Point2D<i32, PixelSize>,
    ) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.move_cursor_to_point(point, &inner.rope, inner.tab_width, &inner.styles);
    }

    pub(crate) fn reload(&mut self) -> IOResult<()> {
        self.0.borrow_mut().reload()
    }

    // FIXME: Spawn thread to write to file
    pub(crate) fn write_file(&mut self, optpath: Option<&str>) -> IOResult<()> {
        self.0.borrow_mut().write_file(optpath)
    }

    pub(super) fn create_empty(bed_handle: BufferBedHandle, ts_core: Rc<TsCore>) -> BufferHandle {
        BufferHandle(Rc::new(RefCell::new(Buffer::empty(bed_handle, ts_core))))
    }

    pub(super) fn create_from_file(
        path: &AbsPath,
        bed_handle: BufferBedHandle,
        ts_core: Rc<TsCore>,
    ) -> IOResult<BufferHandle> {
        Buffer::from_file(path, bed_handle, ts_core)
            .map(|buf| BufferHandle(Rc::new(RefCell::new(buf))))
    }
}

pub(crate) struct Buffer {
    views: FnvHashMap<BufferViewId, View>,
    bed_handle: BufferBedHandle,
    rope: Rope,
    tab_width: usize,
    optpath: Option<AbsPath>,
    // Tree-sitter stuff
    ts_core: Rc<TsCore>,
    optlanguage: Option<Language>,
    opttslang: Option<TsLang>,
    opttree: Option<Tree>,
    // Highlighting
    styles: StyleRanges,
}

impl Buffer {
    // -------- Get next cursor position ---------
    fn cursor_up(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let mut cursor = cursor.clone();
        if cursor.line_num == 0 {
            cursor.reset();
        } else {
            if n < cursor.line_num {
                cursor.line_num -= n;
            } else {
                cursor.line_num = 0;
            }
            cursor.sync_global_x(&self.rope, self.tab_width);
        }
        cursor
    }

    fn cursor_down(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let mut cursor = cursor.clone();
        cursor.line_num += n;
        if cursor.line_num >= self.rope.len_lines() {
            cursor.cidx = self.rope.len_chars();
            cursor.sync_and_update_char_idx_left(&self.rope, self.tab_width);
        } else {
            cursor.sync_global_x(&self.rope, self.tab_width);
        }
        cursor
    }

    fn cursor_left(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let mut cursor = cursor.clone();
        if cursor.line_cidx < n {
            cursor.line_cidx = 0;
        } else {
            cursor.line_cidx -= n;
        }
        cursor.sync_line_cidx_gidx_left(&self.rope, self.tab_width);
        cursor
    }

    fn cursor_right(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let mut cursor = cursor.clone();
        let len_chars = rope_trim_newlines(self.rope.line(cursor.line_num)).len_chars();
        cursor.line_cidx = min(len_chars, cursor.line_cidx + n);
        cursor.sync_line_cidx_gidx_left(&self.rope, self.tab_width);
        cursor
    }

    fn cursor_line_start(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        assert!(n > 0);
        let mut cursor = cursor.clone();
        cursor.line_cidx = 0;
        if cursor.line_num <= n - 1 {
            cursor.line_num = 0;
        } else {
            cursor.line_num -= n - 1;
        }
        cursor.sync_line_cidx_gidx_left(&self.rope, self.tab_width);
        cursor
    }

    fn cursor_line_end(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        assert!(n > 0);
        let mut cursor = cursor.clone();
        cursor.line_num += n - 1;
        if cursor.line_num >= self.rope.len_lines() {
            cursor.cidx = self.rope.len_chars();
            cursor.sync_and_update_char_idx_left(&self.rope, self.tab_width);
        } else {
            cursor.line_cidx = rope_trim_newlines(self.rope.line(cursor.line_num)).len_chars();
            cursor.sync_line_cidx_gidx_left(&self.rope, self.tab_width);
        }
        cursor
    }

    fn cursor_line(&self, cursor: &ViewCursor, linum: usize) -> ViewCursor {
        let mut cursor = cursor.clone();
        cursor.line_num = linum;
        if linum >= self.rope.len_lines() {
            cursor.line_num = self.rope.len_lines() - 1;
        }
        cursor.line_cidx = 0;
        cursor.sync_line_cidx_gidx_left(&self.rope, self.tab_width);
        cursor
    }

    fn cursor_word(&self, cursor: &ViewCursor, mut n: usize, ext: bool) -> ViewCursor {
        let mut cursor = cursor.clone();
        while n > 0 && cursor.cidx <= self.rope.len_chars() {
            if let Some(range) = word_containing(&self.rope, cursor.cidx, ext) {
                cursor.cidx = range.end;
            }
            if let Some(range) = space_containing(&self.rope, cursor.cidx) {
                cursor.cidx = range.end;
            }
            n -= 1;
        }
        cursor.sync_and_update_char_idx_left(&self.rope, self.tab_width);
        cursor
    }

    fn cursor_word_end(&self, cursor: &ViewCursor, mut n: usize, ext: bool) -> ViewCursor {
        assert!(n > 0);
        let mut cursor = cursor.clone();
        if let Some(range) = word_containing(&self.rope, cursor.cidx, ext) {
            if cursor.cidx + 1 < range.end {
                n -= 1;
            }
            cursor.cidx = range.end - 1;
        }
        while n > 0 && cursor.cidx <= self.rope.len_chars() {
            if cursor.cidx < self.rope.len_chars() {
                cursor.cidx += 1;
            }
            if let Some(range) = space_containing(&self.rope, cursor.cidx) {
                cursor.cidx = range.end;
            }
            if let Some(range) = word_containing(&self.rope, cursor.cidx, ext) {
                cursor.cidx = range.end - 1;
            }
            n -= 1;
        }
        cursor.sync_and_update_char_idx_left(&self.rope, self.tab_width);
        cursor
    }

    fn cursor_back(&self, cursor: &ViewCursor, mut n: usize, ext: bool) -> ViewCursor {
        assert!(n > 0);
        let mut cursor = cursor.clone();
        if let Some(range) = word_containing(&self.rope, cursor.cidx, ext) {
            if cursor.cidx > range.start {
                n -= 1;
            }
            cursor.cidx = range.start;
        }
        while n > 0 && cursor.cidx <= self.rope.len_chars() {
            if cursor.cidx > 0 {
                cursor.cidx -= 1;
            }
            if let Some(range) = space_containing(&self.rope, cursor.cidx) {
                cursor.cidx = range.start;
                if cursor.cidx > 0 {
                    cursor.cidx -= 1;
                }
            }
            if let Some(range) = word_containing(&self.rope, cursor.cidx, ext) {
                cursor.cidx = range.start;
            }
            n -= 1;
        }
        cursor.sync_and_update_char_idx_left(&self.rope, self.tab_width);
        cursor
    }

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

    pub(crate) fn move_view_cursor_to_line(&mut self, view_id: &BufferViewId, linum: usize) {
        self.view_mut(view_id).cursor = self.cursor_line(&self.view(view_id).cursor, linum);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn move_view_cursor_to_last_line(&mut self, view_id: &BufferViewId) {
        self.move_view_cursor_to_line(view_id, self.rope.len_lines() - 1);
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
        let view = self.views.get_mut(view_id).unwrap();
        view.cursor.style = style;
        view.cursor
            .sync_line_cidx_gidx_left(&self.rope, self.tab_width);
        self.bed_handle.request_redraw();
    }

    pub(crate) fn snap_to_cursor(&mut self, view_id: &BufferViewId, update_global_x: bool) {
        let view = self.views.get_mut(view_id).unwrap();
        view.snap_to_cursor(&self.rope, self.tab_width, &self.styles, update_global_x);
    }

    // -------- Editing --------
    pub(crate) fn insert_char(&mut self, view_id: &BufferViewId, c: char) {
        let view = self.views.get_mut(view_id).unwrap();
        let cidx = view.cursor.cidx;
        let old_rope = self.rope.clone();
        self.rope.insert_char(cidx, c);
        let fgcol = self.bed_handle.theme().textview.foreground;
        self.styles.insert_default(cidx, 1, fgcol);
        self.edit_tree(old_rope, cidx..cidx, 1);
        for view in self.views.values_mut() {
            if view.cursor.cidx >= cidx {
                view.cursor.cidx += 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.rope, self.tab_width);
            }
        }
        self.bed_handle.request_redraw();
    }

    fn delete_range(&mut self, range: Range<usize>) {
        let old_rope = self.rope.clone();
        self.rope.remove(range.clone());
        self.styles.remove(range.clone());
        self.edit_tree(old_rope, range.clone(), 0);
        for view in self.views.values_mut() {
            if view.cursor.cidx >= range.end {
                view.cursor.cidx -= range.len();
            } else if view.cursor.cidx > range.start {
                view.cursor.cidx = range.start;
            }
            if view.cursor.cidx >= range.start {
                view.cursor
                    .sync_and_update_char_idx_left(&self.rope, self.tab_width);
            }
        }
        self.bed_handle.request_redraw();
    }

    pub(crate) fn delete_left(&mut self, view_id: &BufferViewId, mut n: usize) {
        let view = self.views.get(view_id).unwrap();
        let cidx = view.cursor.cidx;
        if cidx < n {
            n = cidx;
        }
        let start_cidx = cidx - n;
        self.delete_range(start_cidx..cidx);
    }

    pub(crate) fn delete_right(&mut self, view_id: &BufferViewId, mut n: usize) {
        let view = self.views.get(view_id).unwrap();
        let mut cidx = view.cursor.cidx;
        let pre_len_chars = self.rope.len_chars();
        assert!(cidx <= pre_len_chars);
        if cidx == pre_len_chars {
            cidx = pre_len_chars - 1;
            n = 1;
        } else if cidx + n >= pre_len_chars {
            n = pre_len_chars - cidx;
        }
        let end_cidx = cidx + n;
        self.delete_range(cidx..end_cidx);
    }

    pub(crate) fn delete_up(&mut self, view_id: &BufferViewId, mut n: usize) {
        let view = self.views.get(view_id).unwrap();
        if view.cursor.line_num < n {
            n = view.cursor.line_num;
        }
        self.move_view_cursor_up(view_id, n);
        self.delete_down(view_id, n);
    }

    pub(crate) fn delete_down(&mut self, view_id: &BufferViewId, mut n: usize) {
        let view = self.views.get(view_id).unwrap();
        if view.cursor.line_num + n > self.rope.len_lines() {
            n = self.rope.len_lines() - view.cursor.line_num;
        }
        let start_cidx = self.rope.line_to_char(view.cursor.line_num);
        let end_cidx = if view.cursor.line_num + n == self.rope.len_lines() {
            self.rope.len_chars()
        } else {
            self.rope.line_to_char(view.cursor.line_num + n + 1)
        };
        self.delete_range(start_cidx..end_cidx);
    }

    pub(crate) fn delete_to_line_start(&mut self, view_id: &BufferViewId, n: usize) {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_line_start(&view.cursor, n);
            dest.cidx..view.cursor.cidx
        };
        self.delete_range(range);
    }

    pub(crate) fn delete_to_line_end(&mut self, view_id: &BufferViewId, n: usize) {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_line_end(&view.cursor, n);
            view.cursor.cidx..min(dest.cidx, self.rope.len_chars())
        };
        self.delete_range(range);
    }

    pub(crate) fn delete_to_line(&mut self, view_id: &BufferViewId, linum: usize) {
        let view = self.view(view_id);
        if view.cursor.line_num < linum {
            let diff = linum - view.cursor.line_num;
            self.delete_down(view_id, diff);
        } else {
            let diff = view.cursor.line_num - linum;
            self.delete_up(view_id, diff);
        }
    }

    pub(crate) fn delete_to_last_line(&mut self, view_id: &BufferViewId) {
        self.delete_down(
            view_id,
            self.rope.len_lines() - self.view(view_id).cursor.line_num,
        );
    }

    pub(crate) fn delete_word(&mut self, view_id: &BufferViewId, n: usize) {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_word(&view.cursor, n, false);
            view.cursor.cidx..dest.cidx
        };
        self.delete_range(range);
    }

    pub(crate) fn delete_word_extended(&mut self, view_id: &BufferViewId, n: usize) {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_word(&view.cursor, n, true);
            view.cursor.cidx..dest.cidx
        };
        self.delete_range(range);
    }

    pub(crate) fn delete_word_end(&mut self, view_id: &BufferViewId, n: usize) {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_word_end(&view.cursor, n, false);
            view.cursor.cidx..min(dest.cidx + 1, self.rope.len_chars())
        };
        self.delete_range(range);
    }

    pub(crate) fn delete_word_end_extended(&mut self, view_id: &BufferViewId, n: usize) {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_word_end(&view.cursor, n, true);
            view.cursor.cidx..min(dest.cidx + 1, self.rope.len_chars())
        };
        self.delete_range(range);
    }

    pub(crate) fn delete_back(&mut self, view_id: &BufferViewId, n: usize) {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_back(&view.cursor, n, false);
            dest.cidx..view.cursor.cidx
        };
        self.delete_range(range);
    }

    pub(crate) fn delete_back_extended(&mut self, view_id: &BufferViewId, n: usize) {
        let range = {
            let view = self.view(view_id);
            let dest = self.cursor_back(&view.cursor, n, true);
            dest.cidx..view.cursor.cidx
        };
        self.delete_range(range);
    }

    pub(crate) fn replace_repeated(&mut self, view_id: &BufferViewId, c: char, n: usize) {
        assert!(n > 0);
        let mut cursor = self.view(view_id).cursor.clone();
        let len_chars = rope_trim_newlines(self.rope.line(cursor.line_num)).len_chars();
        if cursor.line_cidx + n > len_chars {
            return;
        }
        let mut buf = [0; 4];
        let s = c.encode_utf8(&mut buf);
        let old_rope = self.rope.clone();
        self.rope.remove(cursor.cidx..cursor.cidx + n);
        self.rope.insert(cursor.cidx, &s.repeat(n));
        let fgcol = self.bed_handle.theme().textview.foreground;
        self.styles.set_default(cursor.cidx..cursor.cidx + n, fgcol);
        self.edit_tree(old_rope, cursor.cidx..cursor.cidx + n, n);
        cursor.cidx += n - 1;
        cursor.sync_and_update_char_idx_left(&self.rope, self.tab_width);
        self.view_mut(view_id).cursor = cursor;
    }

    pub(crate) fn update_text_size(&mut self, view_id: &BufferViewId, diff: i16) {
        self.view_mut(view_id).update_text_size(diff);
        self.bed_handle.request_redraw();
    }

    // -------- Internal stuff --------
    fn empty(bed_handle: BufferBedHandle, ts_core: Rc<TsCore>) -> Buffer {
        Buffer {
            views: FnvHashMap::default(),
            rope: Rope::new(),
            bed_handle,
            tab_width: 8,
            optpath: None,
            ts_core,
            optlanguage: None,
            opttslang: None,
            opttree: None,
            styles: StyleRanges::new(),
        }
    }

    fn from_file(
        path: &AbsPath,
        bed_handle: BufferBedHandle,
        ts_core: Rc<TsCore>,
    ) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| {
                let (optlanguage, opttslang) = path
                    .as_ref()
                    .extension()
                    .and_then(|s| s.to_str())
                    .and_then(|s| ts_core.parser_from_extension(s))
                    .map(|(l, t)| (Some(l), Some(t)))
                    .unwrap_or((None, None));
                let mut ret = Buffer {
                    rope,
                    bed_handle,
                    views: FnvHashMap::default(),
                    tab_width: 8,
                    optpath: Some(path.clone()),
                    ts_core,
                    optlanguage,
                    opttslang,
                    opttree: None,
                    styles: StyleRanges::new(),
                };
                let fgcol = ret.bed_handle.theme().textview.foreground;
                ret.styles.insert_default(0, ret.rope.len_chars(), fgcol);
                ret.recreate_parse_tree();
                ret
            })
    }

    fn reload(&mut self) -> IOResult<()> {
        if let Some(path) = &self.optpath {
            File::open(path)
                .and_then(|f| Rope::from_reader(f))
                .map(|rope| {
                    let old_len_chars = self.rope.len_chars();
                    self.rope = rope;
                    for view in self.views.values_mut() {
                        view.scroll_to_top();
                    }
                    let fgcol = self.bed_handle.theme().textview.foreground;
                    self.styles.remove(0..old_len_chars);
                    self.styles.insert_default(0, self.rope.len_chars(), fgcol);
                    self.recreate_parse_tree();
                })
        } else {
            // FIXME: Print some error?
            Ok(())
        }
    }

    fn view(&self, view_id: &BufferViewId) -> &View {
        self.views.get(view_id).unwrap()
    }

    fn view_mut(&mut self, view_id: &BufferViewId) -> &mut View {
        self.views.get_mut(view_id).unwrap()
    }

    fn write_file(&mut self, optpath: Option<&str>) -> IOResult<()> {
        if let Some(path) = optpath
            .map(|s| AbsPath::from(s))
            .or(self.optpath.as_ref().map(|s| s.clone()))
        {
            let mut f = File::create(&path)?;
            for c in self.rope.chunks() {
                f.write(c.as_bytes())?;
            }
            if self.optpath.is_none() {
                let (optlanguage, opttslang) = path
                    .as_ref()
                    .extension()
                    .and_then(|s| s.to_str())
                    .and_then(|s| self.ts_core.parser_from_extension(s))
                    .map(|(l, t)| (Some(l), Some(t)))
                    .unwrap_or((None, None));
                self.optpath = Some(path.clone());
                self.optlanguage = optlanguage;
                self.opttslang = opttslang;
                self.recreate_parse_tree();
            }
            Ok(())
        } else {
            // FIXME: Feedback in bed UI
            eprintln!("ERROR: No file specified");
            Ok(())
        }
    }

    fn recreate_parse_tree(&mut self) {
        let rope = &self.rope;
        if let Some(tslang) = &mut self.opttslang {
            let tree = tslang
                .parser
                .parse_with(
                    &mut |boff, _| {
                        if boff >= rope.len_bytes() {
                            ""
                        } else {
                            let (chunk, chunk_byte_idx, _, _) = rope.chunk_at_byte(boff);
                            &chunk[boff - chunk_byte_idx..]
                        }
                    },
                    None,
                )
                .expect("failed to parse");
            /*
            {
                let mut cursor = tree.walk();
                walk_recur(&mut cursor, 0);
            }
            */
            self.opttree = Some(tree);
            self.rehighlight_range(tree_sitter::Range {
                start_byte: 0,
                end_byte: self.rope.len_bytes(),
                start_point: tree_sitter::Point::new(0, 0),
                end_point: tree_sitter::Point::new(
                    self.rope.len_lines(),
                    self.rope.len_bytes() - self.rope.line_to_byte(self.rope.len_lines()),
                ),
            })
        }
    }

    fn edit_tree(&mut self, old_rope: Rope, old_crange: Range<usize>, new_clen: usize) {
        let rope = &self.rope;
        if let Some(mut old_tree) = self.opttree.take() {
            let start_byte = old_rope.char_to_byte(old_crange.start);
            let old_end_byte = old_rope.char_to_byte(old_crange.end);
            let new_end_byte = old_rope.char_to_byte(old_crange.start + new_clen);
            let start_linum = old_rope.byte_to_line(start_byte);
            let start_linoff = start_byte - old_rope.line_to_byte(start_linum);
            let old_end_linum = old_rope.byte_to_line(old_end_byte);
            let old_end_linoff = old_end_byte - old_rope.line_to_byte(old_end_linum);
            let new_end_linum = old_rope.byte_to_line(new_end_byte);
            let new_end_linoff = new_end_byte - old_rope.line_to_byte(new_end_linum);
            old_tree.edit(&InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position: TsPoint::new(start_linum, start_linoff),
                old_end_position: TsPoint::new(old_end_linum, old_end_linoff),
                new_end_position: TsPoint::new(new_end_linum, new_end_linoff),
            });
            if let Some(tslang) = &mut self.opttslang {
                let new_tree = tslang
                    .parser
                    .parse_with(
                        &mut |boff, _| {
                            if boff >= rope.len_bytes() {
                                ""
                            } else {
                                let (chunk, chunk_byte_idx, _, _) = rope.chunk_at_byte(boff);
                                &chunk[boff - chunk_byte_idx..]
                            }
                        },
                        Some(&mut old_tree),
                    )
                    .expect("failed to parse");
                self.opttree = Some(new_tree.clone());
                for range in old_tree.changed_ranges(&new_tree) {
                    self.rehighlight_range(range);
                }
            }
        }
    }

    fn rehighlight_range(&mut self, range: tree_sitter::Range) {
        if self.opttree.is_none() || self.opttslang.is_none() {
            return;
        }
        let tree = self.opttree.as_ref().unwrap();
        let tslang = self.opttslang.as_ref().unwrap();
        let rope = &self.rope;
        let theme = self.bed_handle.theme();
        // Reset highlighting
        let crange = rope.byte_to_char(range.start_byte)..rope.byte_to_char(range.end_byte);
        self.styles.set_default(crange, theme.textview.foreground);
        // Add new highlighting
        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(range.start_byte, range.end_byte);
        let iter = cursor.captures(&tslang.hl_query, tree.root_node(), |node| {
            let range = node.byte_range();
            let range = rope.byte_to_char(range.start)..rope.byte_to_char(range.end);
            rope.slice(range).to_string()
        });
        let mut buf = String::new();
        for (query_match, _) in iter {
            for capture in query_match.captures {
                let range = capture.node.byte_range();
                let crange = rope.byte_to_char(range.start)..rope.byte_to_char(range.end);
                buf.clear();
                let mut elem = None;
                let capture_name = &tslang.hl_query.capture_names()[capture.index as usize];
                for split in capture_name.split('.') {
                    if buf.len() > 0 {
                        buf.push('.');
                    }
                    buf.push_str(split);
                    if let Some(se) = theme.syntax.get(&buf) {
                        elem = Some(se);
                    }
                }
                if let Some(elem) = elem {
                    let style = TextStyle::new(elem.weight, elem.slant);
                    self.styles.set_style(crange.clone(), style);
                    self.styles.set_color(crange, elem.foreground);
                }
            }
        }
    }
}

#[allow(dead_code)]
fn walk_recur(cursor: &mut tree_sitter::TreeCursor, indent: usize) {
    eprint!("\n{}({}", " ".repeat(indent), cursor.node().kind());
    if cursor.goto_first_child() {
        walk_recur(cursor, indent + 2);
    }
    eprint!(")");
    while cursor.goto_next_sibling() {
        eprint!("\n{}({}", " ".repeat(indent), cursor.node().kind());
        if cursor.goto_first_child() {
            walk_recur(cursor, indent + 2);
        }
        eprint!(")");
    }
    cursor.goto_parent();
}
