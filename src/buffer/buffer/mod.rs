// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::{Ref, RefCell, RefMut};
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
use crate::ts::{TsCore, TsLang};

use super::rope_stuff::{space_containing, word_containing};
use super::view::{View, ViewCursor};
use super::{BufferBedHandle, BufferViewId};

mod editing;
mod movement;

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
        let view = View::new(inner.bed_handle.clone(), rect, inner.shared.clone());
        inner.views.insert(view_id.clone(), view);
    }

    pub(crate) fn set_view_rect(&mut self, view_id: &BufferViewId, rect: Rect<u32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.set_rect(rect);
    }

    pub(crate) fn draw_view(&mut self, view_id: &BufferViewId, painter: &mut Painter) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.draw(painter);
    }

    pub(crate) fn scroll_view(&mut self, view_id: &BufferViewId, scroll: Vector2D<i32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.scroll(scroll);
    }

    pub(crate) fn move_view_cursor_to_point(
        &mut self,
        view_id: &BufferViewId,
        point: Point2D<i32, PixelSize>,
    ) {
        let inner = &mut *self.0.borrow_mut();
        let view = inner.views.get_mut(view_id).unwrap();
        view.move_cursor_to_point(point);
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

#[derive(Clone)]
pub(super) struct BufferSharedState(Rc<RefCell<BufferSharedStateInner>>);

impl BufferSharedState {
    pub(super) fn borrow(&self) -> Ref<BufferSharedStateInner> {
        self.0.borrow()
    }

    fn borrow_mut(&mut self) -> RefMut<BufferSharedStateInner> {
        self.0.borrow_mut()
    }
}

pub(super) struct BufferSharedStateInner {
    pub(super) rope: Rope,
    pub(super) tab_width: usize,
    pub(super) indent_tabs: bool,
    pub(super) optname: Option<String>,
    pub(super) optlanguage: Option<Language>,
    pub(super) styles: StyleRanges,
}

pub(crate) struct Buffer {
    views: FnvHashMap<BufferViewId, View>,
    bed_handle: BufferBedHandle,
    optpath: Option<AbsPath>,
    // Tree-sitter stuff
    ts_core: Rc<TsCore>,
    opttslang: Option<TsLang>,
    opttree: Option<Tree>,
    // Shared state with all views
    shared: BufferSharedState,
}

impl Buffer {
    // -------- Get next cursor position ---------
    fn cursor_up(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let shared = self.shared.borrow();
        let mut cursor = cursor.clone();
        if cursor.line_num == 0 {
            cursor.reset();
        } else {
            if n < cursor.line_num {
                cursor.line_num -= n;
            } else {
                cursor.line_num = 0;
            }
            cursor.sync_global_x(&shared.rope, shared.tab_width);
        }
        cursor
    }

    fn cursor_down(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let shared = self.shared.borrow();
        let mut cursor = cursor.clone();
        cursor.line_num += n;
        if cursor.line_num >= shared.rope.len_lines() {
            cursor.cidx = shared.rope.len_chars();
            cursor.sync_and_update_char_idx_left(&shared.rope, shared.tab_width);
        } else {
            cursor.sync_global_x(&shared.rope, shared.tab_width);
        }
        cursor
    }

    fn cursor_left(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let shared = self.shared.borrow();
        let mut cursor = cursor.clone();
        if cursor.line_cidx < n {
            cursor.line_cidx = 0;
        } else {
            cursor.line_cidx -= n;
        }
        cursor.sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
        cursor
    }

    fn cursor_right(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let shared = self.shared.borrow();
        let mut cursor = cursor.clone();
        let len_chars = rope_trim_newlines(shared.rope.line(cursor.line_num)).len_chars();
        cursor.line_cidx = min(len_chars, cursor.line_cidx + n);
        cursor.sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
        cursor
    }

    fn cursor_line_start(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let shared = self.shared.borrow();
        assert!(n > 0);
        let mut cursor = cursor.clone();
        cursor.line_cidx = 0;
        if cursor.line_num <= n - 1 {
            cursor.line_num = 0;
        } else {
            cursor.line_num -= n - 1;
        }
        cursor.sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
        cursor
    }

    fn cursor_line_end(&self, cursor: &ViewCursor, n: usize) -> ViewCursor {
        let shared = self.shared.borrow();
        assert!(n > 0);
        let mut cursor = cursor.clone();
        cursor.line_num += n - 1;
        if cursor.line_num >= shared.rope.len_lines() {
            cursor.cidx = shared.rope.len_chars();
            cursor.sync_and_update_char_idx_left(&shared.rope, shared.tab_width);
        } else {
            cursor.line_cidx = rope_trim_newlines(shared.rope.line(cursor.line_num)).len_chars();
            cursor.sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
        }
        cursor
    }

    fn cursor_first_non_blank(&self, cursor: &ViewCursor) -> ViewCursor {
        let shared = self.shared.borrow();
        let mut cursor = cursor.clone();
        for (i, c) in shared.rope.line(cursor.line_num).chars().enumerate() {
            if !c.is_whitespace() {
                cursor.line_cidx = i;
                cursor.sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
                break;
            }
        }
        cursor
    }

    fn cursor_line(&self, cursor: &ViewCursor, linum: usize) -> ViewCursor {
        let shared = self.shared.borrow();
        let mut cursor = cursor.clone();
        cursor.line_num = linum;
        if linum >= shared.rope.len_lines() {
            cursor.line_num = shared.rope.len_lines() - 1;
        }
        cursor.line_cidx = 0;
        cursor.sync_line_cidx_gidx_left(&shared.rope, shared.tab_width);
        cursor
    }

    fn cursor_word(&self, cursor: &ViewCursor, mut n: usize, ext: bool) -> ViewCursor {
        let shared = self.shared.borrow();
        let mut cursor = cursor.clone();
        while n > 0 && cursor.cidx <= shared.rope.len_chars() {
            if let Some(range) = word_containing(&shared.rope, cursor.cidx, ext) {
                cursor.cidx = range.end;
            }
            if let Some(range) = space_containing(&shared.rope, cursor.cidx) {
                cursor.cidx = range.end;
            }
            n -= 1;
        }
        cursor.sync_and_update_char_idx_left(&shared.rope, shared.tab_width);
        cursor
    }

    fn cursor_word_end(&self, cursor: &ViewCursor, mut n: usize, ext: bool) -> ViewCursor {
        let shared = self.shared.borrow();
        assert!(n > 0);
        let mut cursor = cursor.clone();
        if let Some(range) = word_containing(&shared.rope, cursor.cidx, ext) {
            if cursor.cidx + 1 < range.end {
                n -= 1;
            }
            cursor.cidx = range.end - 1;
        }
        while n > 0 && cursor.cidx <= shared.rope.len_chars() {
            if cursor.cidx < shared.rope.len_chars() {
                cursor.cidx += 1;
            }
            if let Some(range) = space_containing(&shared.rope, cursor.cidx) {
                cursor.cidx = range.end;
            }
            if let Some(range) = word_containing(&shared.rope, cursor.cidx, ext) {
                cursor.cidx = range.end - 1;
            }
            n -= 1;
        }
        cursor.sync_and_update_char_idx_left(&shared.rope, shared.tab_width);
        cursor
    }

    fn cursor_back(&self, cursor: &ViewCursor, mut n: usize, ext: bool) -> ViewCursor {
        let shared = self.shared.borrow();
        assert!(n > 0);
        let mut cursor = cursor.clone();
        if let Some(range) = word_containing(&shared.rope, cursor.cidx, ext) {
            if cursor.cidx > range.start {
                n -= 1;
            }
            cursor.cidx = range.start;
        }
        while n > 0 && cursor.cidx <= shared.rope.len_chars() {
            if cursor.cidx > 0 {
                cursor.cidx -= 1;
            }
            if let Some(range) = space_containing(&shared.rope, cursor.cidx) {
                cursor.cidx = range.start;
                if cursor.cidx > 0 {
                    cursor.cidx -= 1;
                }
            }
            if let Some(range) = word_containing(&shared.rope, cursor.cidx, ext) {
                cursor.cidx = range.start;
            }
            n -= 1;
        }
        cursor.sync_and_update_char_idx_left(&shared.rope, shared.tab_width);
        cursor
    }

    // -------- Updating view state --------
    pub(crate) fn set_mode(&mut self, view_id: &BufferViewId, mode: super::Mode) {
        self.view_mut(view_id).set_mode(mode);
    }

    // -------- Internal stuff --------
    fn empty(bed_handle: BufferBedHandle, ts_core: Rc<TsCore>) -> Buffer {
        let (tab_width, indent_tabs) = {
            let config = bed_handle.config();
            (config.tab_width, config.indent_tabs)
        };
        Buffer {
            views: FnvHashMap::default(),
            bed_handle,
            optpath: None,
            ts_core,
            opttslang: None,
            opttree: None,
            shared: BufferSharedState(Rc::new(RefCell::new(BufferSharedStateInner {
                rope: Rope::new(),
                tab_width,
                indent_tabs,
                optlanguage: None,
                optname: None,
                styles: StyleRanges::new(),
            }))),
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
                let (mut tab_width, mut indent_tabs) = {
                    let config = bed_handle.config();
                    (config.tab_width, config.indent_tabs)
                };
                let (optlanguage, opttslang) = path
                    .as_ref()
                    .extension()
                    .and_then(|s| s.to_str())
                    .and_then(|s| ts_core.parser_from_extension(s))
                    .map(|(l, t)| (Some(l), Some(t)))
                    .unwrap_or((None, None));
                if let Some(lang) = &optlanguage {
                    let config = bed_handle.config();
                    config.language.get(lang).map(|lang| {
                        tab_width = lang.tab_width;
                        indent_tabs = lang.indent_tabs;
                    });
                }
                let mut ret = Buffer {
                    bed_handle,
                    views: FnvHashMap::default(),
                    optpath: Some(path.clone()),
                    ts_core,
                    opttslang,
                    opttree: None,
                    shared: BufferSharedState(Rc::new(RefCell::new(BufferSharedStateInner {
                        rope,
                        tab_width,
                        indent_tabs,
                        optlanguage,
                        optname: path.file_name().map(|s| s.to_owned()),
                        styles: StyleRanges::new(),
                    }))),
                };
                let fgcol = ret.bed_handle.theme().textview.foreground;
                {
                    let shared = &mut *ret.shared.borrow_mut();
                    shared
                        .styles
                        .insert_default(0, shared.rope.len_chars(), fgcol);
                }
                ret.recreate_parse_tree();
                ret
            })
    }

    fn reload(&mut self) -> IOResult<()> {
        if let Some(path) = &self.optpath {
            File::open(path)
                .and_then(|f| Rope::from_reader(f))
                .map(|rope| {
                    {
                        let shared = &mut *self.shared.borrow_mut();
                        let old_len_chars = shared.rope.len_chars();
                        shared.rope = rope;
                        let fgcol = self.bed_handle.theme().textview.foreground;
                        shared.styles.remove(0..old_len_chars);
                        shared
                            .styles
                            .insert_default(0, shared.rope.len_chars(), fgcol);
                    }
                    self.recreate_parse_tree();
                    for view in self.views.values_mut() {
                        view.scroll_to_top();
                    }
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
            for c in self.shared.borrow().rope.chunks() {
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
                {
                    let mut shared = self.shared.borrow_mut();
                    if let Some(lang) = &optlanguage {
                        let config = self.bed_handle.config();
                        config.language.get(lang).map(|lang| {
                            shared.tab_width = lang.tab_width;
                            shared.indent_tabs = lang.indent_tabs;
                        });
                    }
                    shared.optlanguage = optlanguage;
                    shared.optname = path.file_name().map(|s| s.to_owned());
                }
                self.optpath = Some(path.clone());
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
        let rope = self.shared.borrow().rope.clone();
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
            self.opttree = Some(tree);
            self.rehighlight_range(0..rope.len_bytes())
        }
    }

    fn edit_tree(&mut self, old_rope: Rope, old_crange: Range<usize>, new_clen: usize) {
        let rope = self.shared.borrow().rope.clone();
        if let Some(mut old_tree) = self.opttree.take() {
            let start_byte = old_rope.char_to_byte(old_crange.start);
            let old_end_byte = old_rope.char_to_byte(old_crange.end);
            let new_end_byte = rope.char_to_byte(old_crange.start + new_clen);
            let start_linum = old_rope.byte_to_line(start_byte);
            let start_linoff = start_byte - old_rope.line_to_byte(start_linum);
            let old_end_linum = old_rope.byte_to_line(old_end_byte);
            let old_end_linoff = old_end_byte - old_rope.line_to_byte(old_end_linum);
            let new_end_linum = rope.byte_to_line(new_end_byte);
            let new_end_linoff = new_end_byte - rope.line_to_byte(new_end_linum);
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
                let start = rope.line_to_byte(start_linum);
                let end = rope.line_to_byte(new_end_linum) + rope.line(new_end_linum).len_bytes();
                self.rehighlight_range(start..end);
                for range in old_tree.changed_ranges(&new_tree) {
                    self.rehighlight_range(range.start_byte..range.end_byte);
                }
                /*
                {
                    eprint!("\n\n******** NEW TREE ********");
                    let mut cursor = new_tree.walk();
                    walk_recur(&mut cursor, 0);
                }
                */
            }
        }
    }

    fn rehighlight_range(&mut self, byte_range: Range<usize>) {
        if self.opttree.is_none() || self.opttslang.is_none() {
            return;
        }
        let shared = &mut *self.shared.borrow_mut();
        let tree = self.opttree.as_ref().unwrap();
        let tslang = self.opttslang.as_ref().unwrap();
        let rope = &shared.rope;
        let theme = self.bed_handle.theme();
        // Reset highlighting
        let crange = rope.byte_to_char(byte_range.start)..rope.byte_to_char(byte_range.end);
        shared.styles.set_default(crange, theme.textview.foreground);
        // Add new highlighting
        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(byte_range.start, byte_range.end);
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
                if capture_name == "conceal" {
                    shared.styles.set_conceal(crange.clone(), true);
                    continue;
                } else {
                    shared.styles.set_conceal(crange.clone(), false);
                }
                if capture_name.starts_with("_") {
                    continue;
                }
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
                    shared.styles.set_style(crange.clone(), style);
                    shared.styles.set_color(crange.clone(), elem.foreground);
                    shared.styles.set_under(crange.clone(), elem.underline);
                    shared.styles.set_scale(crange, elem.scale);
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