// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fs::File;
use std::io::Result as IOResult;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use euclid::{Rect, Size2D};
use fnv::FnvHashMap;
use ropey::Rope;

use crate::common::{rope_trim_newlines, PixelSize, DPI};
use crate::font::FaceKey;
use crate::painter::WidgetPainter;
use crate::style::{Color, TextSize, TextStyle};
use crate::text::{ShapedText, TextShaper};

use super::view::BufferView;
use super::BufferViewID;

const SYNC_HL_LINES: usize = 1000;

pub(crate) struct Buffer {
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
    tab_width: usize,
    // Text rendering
    text_shaper: Arc<Mutex<TextShaper>>,
    face_key: FaceKey,
    text_size: TextSize,
    dpi: Size2D<u32, DPI>,
    shaped_lines: Arc<Mutex<Vec<ShapedText>>>,
}

impl Buffer {
    // -------- View management ----------------
    pub(crate) fn new_view(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views.insert(id.clone(), BufferView::new(rect));
    }

    pub(crate) fn set_view_rect(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        let view = self.views.get_mut(id).unwrap();
        view.rect = rect;
        // TODO: Ensure we've shaped till here
        view.snap_to_cursor(&self.shaped_lines.lock().unwrap());
    }

    pub(crate) fn draw_view(&self, id: &BufferViewID, painter: &mut WidgetPainter) {
        let view = self.views.get(id).unwrap();
        let mut shaper = self.text_shaper.lock().unwrap();
        let shaped_lines = self.shaped_lines.lock().unwrap();
        view.draw(&shaped_lines, &mut shaper, painter);
    }

    pub(crate) fn remove_view(&mut self, id: &BufferViewID) {
        self.views.remove(id);
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
        // TODO: Ensure we've shaped till here
        view.snap_to_cursor(&self.shaped_lines.lock().unwrap());
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
        // TODO: Ensure we've shaped till here
        view.snap_to_cursor(&self.shaped_lines.lock().unwrap());
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
        // TODO: Ensure we've shaped till here
        view.snap_to_cursor(&self.shaped_lines.lock().unwrap());
    }

    pub(crate) fn move_view_cursor_right(&mut self, id: &BufferViewID, n: usize) {
        let view = self.views.get_mut(id).unwrap();
        view.cursor.line_cidx += n;
        view.cursor
            .sync_line_cidx_gidx_right(&self.data, self.tab_width);
        // TODO: Ensure we've shaped till here
        view.snap_to_cursor(&self.shaped_lines.lock().unwrap());
    }

    // -------- View edits -----------------

    pub(crate) fn view_insert_char(&mut self, id: &BufferViewID, c: char) {
        let (cidx, linum) = {
            let view = self.views.get_mut(id).unwrap();
            self.data.insert_char(view.cursor.char_idx, c);
            (view.cursor.char_idx, view.cursor.line_num)
        };
        for view in self.views.values_mut() {
            if view.cursor.char_idx >= cidx {
                view.cursor.char_idx += 1;
                view.cursor
                    .sync_and_update_char_idx_left(&self.data, self.tab_width);
                // TODO: Ensure we've shaped till here
                view.snap_to_cursor(&self.shaped_lines.lock().unwrap());
            }
        }
        self.shape_text_from_linum(linum)
    }

    // -------- Create buffer ----------------
    pub(super) fn empty(
        text_shaper: Arc<Mutex<TextShaper>>,
        face_key: FaceKey,
        text_size: TextSize,
        dpi: Size2D<u32, DPI>,
    ) -> Buffer {
        let mut ret = Buffer {
            text_size: text_size,
            face_key: face_key,
            tab_width: 8,
            dpi: dpi,
            text_shaper: text_shaper,
            data: Rope::new(),
            shaped_lines: Arc::new(Mutex::new(Vec::new())),
            views: FnvHashMap::default(),
        };
        ret.shape_text_from_linum(0);
        ret
    }

    pub(super) fn from_file(
        path: &str,
        text_shaper: Arc<Mutex<TextShaper>>,
        face_key: FaceKey,
        text_size: TextSize,
        dpi: Size2D<u32, DPI>,
    ) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| {
                let mut ret = Buffer {
                    text_size: text_size,
                    face_key: face_key,
                    tab_width: 8,
                    dpi: dpi,
                    text_shaper: text_shaper,
                    data: rope,
                    shaped_lines: Arc::new(Mutex::new(Vec::new())),
                    views: FnvHashMap::default(),
                };
                ret.shape_text_from_linum(0);
                ret
            })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| self.data = rope)
    }

    // -------- Shape text ----------------
    fn shape_text_from_linum(&mut self, start_linum: usize) {
        let shaper = Arc::clone(&self.text_shaper);
        let shaped_lines = Arc::clone(&self.shaped_lines);

        let rope = self.data.clone();

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = pair.clone();

        let dpi = self.dpi;
        let tab_width = self.tab_width;
        let face_key = self.face_key;
        let text_size = self.text_size;

        thread::spawn(move || {
            let (lock, cvar) = &*pair2;

            // Shape first SYNC_HL_LINES lines
            {
                let mut linum = start_linum;
                let mut shaper = shaper.lock().unwrap();
                let mut shaped_lines = shaped_lines.lock().unwrap();
                shaped_lines.truncate(start_linum);

                for line in rope.lines() {
                    let trimmed = rope_trim_newlines(line);
                    let len_chars = trimmed.len_chars();
                    let shaped = shaper.shape_line_rope(
                        trimmed,
                        dpi,
                        tab_width,
                        &[(len_chars, face_key)],
                        &[(len_chars, TextStyle::default())],
                        &[(len_chars, text_size)],
                        &[(len_chars, Color::new(0, 0, 0, 0xff))],
                        &[(len_chars, None)],
                    );
                    shaped_lines.push(shaped);
                    linum += 1;
                    if linum >= SYNC_HL_LINES {
                        break;
                    }
                }

                let mut initial_hl_done = lock.lock().unwrap();
                *initial_hl_done = true;
                cvar.notify_one();
            }

            // Shape the rest of the text
            if rope.len_lines() > SYNC_HL_LINES {
                for line in rope.lines_at(SYNC_HL_LINES) {
                    let trimmed = rope_trim_newlines(line);
                    let len_chars = trimmed.len_chars();
                    let shaped = shaper.lock().unwrap().shape_line_rope(
                        trimmed,
                        dpi,
                        tab_width,
                        &[(len_chars, face_key)],
                        &[(len_chars, TextStyle::default())],
                        &[(len_chars, text_size)],
                        &[(len_chars, Color::new(0, 0, 0, 0xff))],
                        &[(len_chars, None)],
                    );
                    shaped_lines.lock().unwrap().push(shaped);
                }
            }
        });

        // Wait for the first few lines to be highlighted
        let (lock, cvar) = &*pair;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
    }
}
