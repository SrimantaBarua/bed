// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use euclid::{point2, size2, Rect};
use unicode_segmentation::UnicodeSegmentation;

use crate::common::PixelSize;
use crate::config::Config;
use crate::painter::Painter;
use crate::style::TextStyle;
use crate::text::{CursorStyle, StyleType, TextAlign, TextCursor};
use crate::theme::ThemeSet;

mod command;

const TAB_WIDTH: usize = 8;

pub(crate) struct Prompt {
    pub(crate) rect: Option<Rect<u32, PixelSize>>,
    pub(crate) needs_redraw: bool,
    config: Rc<RefCell<Config>>,
    theme_set: Rc<ThemeSet>,
    cursor: Cursor,
    text: String,
    prompt_len: usize,
}

impl Prompt {
    pub(crate) fn new(
        window_rect: Rect<u32, PixelSize>,
        config: Rc<RefCell<Config>>,
        theme_set: Rc<ThemeSet>,
    ) -> Prompt {
        let height = min_required_height(&config);
        let rect = if window_rect.height() >= height * 2 {
            Some(Rect::new(
                point2(
                    window_rect.origin.x,
                    window_rect.origin.y + window_rect.height() - height,
                ),
                size2(window_rect.width(), height),
            ))
        } else {
            None
        };
        Prompt {
            rect,
            needs_redraw: true,
            config,
            theme_set,
            cursor: Cursor { bidx: 0, gidx: 0 },
            text: "".to_owned(),
            prompt_len: 0,
        }
    }

    pub(crate) fn resize(&mut self, window_rect: Rect<u32, PixelSize>) {
        let height = min_required_height(&self.config);
        if window_rect.height() >= height * 2 {
            self.rect = Some(Rect::new(
                point2(
                    window_rect.origin.x,
                    window_rect.origin.y + window_rect.height() - height,
                ),
                size2(window_rect.width(), height),
            ));
        } else {
            self.rect = None;
        }
        self.needs_redraw = true;
    }

    pub(super) fn draw(&mut self, painter: &mut Painter) {
        self.needs_redraw = false;
        if let Some(rect) = self.rect {
            let cfg = &mut *self.config.borrow_mut();
            let theme = self.theme_set.get(&cfg.theme);
            let mut paint_ctx = painter.widget_ctx(rect.cast(), theme.prompt.background, false);

            if self.prompt_len > 0 {
                let text_font = &mut cfg.prompt_font;
                let mut text_ctx = text_font.render_ctx(&mut paint_ctx);
                let origin =
                    point2(cfg.prompt_padding_horizontal, cfg.prompt_padding_vertical).cast();
                let width = (rect.width() - cfg.prompt_padding_horizontal * 2) as f32;
                let text_cursor = Some(TextCursor {
                    gidx: self.cursor.gidx,
                    style: CursorStyle::Line,
                    color: theme.prompt.cursor,
                });
                let fgcol = theme.prompt.foreground;
                text_ctx.draw_line(
                    &self.text.as_str(),
                    StyleType::Const(0..self.text.len(), TextStyle::default(), fgcol, false, 1.0),
                    TAB_WIDTH,
                    origin,
                    width,
                    text_cursor,
                    cfg.prompt_font_size,
                    TextAlign::Left,
                );
            }
        }
    }

    pub(crate) fn set_prompt(&mut self, s: &str) {
        self.text.clear();
        self.text.push_str(s);
        self.prompt_len = s.len();
        self.cursor.set_bidx(s.len(), s);
        self.needs_redraw = true;
    }

    pub(crate) fn clear(&mut self) {
        self.text.clear();
        self.prompt_len = 0;
        self.cursor.reset();
        self.needs_redraw = true;
    }

    pub(crate) fn get_command(&mut self) -> Option<String> {
        if self.prompt_len == 0 {
            None
        } else {
            Some(self.text[self.prompt_len..].to_owned())
        }
    }

    pub(crate) fn move_left(&mut self) {
        if let Some(bidx) = self.prev_bidx() {
            self.cursor.set_bidx(bidx, &self.text);
            self.needs_redraw = true;
        }
    }

    pub(crate) fn move_right(&mut self) {
        if let Some(bidx) = self.next_bidx() {
            self.cursor.set_bidx(bidx, &self.text);
            self.needs_redraw = true;
        }
    }

    pub(crate) fn move_start(&mut self) {
        self.cursor.set_bidx(self.prompt_len, &self.text);
        self.needs_redraw = true;
    }

    pub(crate) fn move_end(&mut self) {
        self.cursor.set_bidx(self.text.len(), &self.text);
        self.needs_redraw = true;
    }

    pub(crate) fn delete_left(&mut self) {
        if let Some(bidx) = self.prev_bidx() {
            self.text.replace_range(bidx..self.cursor.bidx, "");
            self.cursor.set_bidx(bidx, &self.text);
            self.needs_redraw = true;
        }
    }

    pub(crate) fn delete_right(&mut self) {
        if let Some(bidx) = self.next_bidx() {
            self.text.replace_range(self.cursor.bidx..bidx, "");
            self.cursor.set_bidx(self.cursor.bidx, &self.text);
            self.needs_redraw = true;
        }
    }

    pub(crate) fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor.bidx, c);
        self.cursor
            .set_bidx(self.cursor.bidx + c.len_utf8(), &self.text);
        self.needs_redraw = true;
    }

    fn prev_bidx(&self) -> Option<usize> {
        let mut cis = self.text[self.prompt_len..self.cursor.bidx].char_indices();
        cis.next_back().map(|(i, _)| self.prompt_len + i)
    }

    fn next_bidx(&self) -> Option<usize> {
        let mut cis = self.text[self.cursor.bidx..].char_indices();
        cis.next().map(|_| {
            cis.next()
                .map(|(i, _)| self.cursor.bidx + i)
                .unwrap_or(self.text.len())
        })
    }
}

fn min_required_height(config: &RefCell<Config>) -> u32 {
    let mut config_ref = config.borrow_mut();
    let font_size = config_ref.prompt_font_size;
    let font_metrics = config_ref.prompt_font.metrics(font_size);
    let height = (font_metrics.ascender - font_metrics.descender).to_f32()
        + (config_ref.prompt_padding_vertical as f32) * 2.0;
    height.ceil() as u32
}

struct Cursor {
    bidx: usize,
    gidx: usize,
}

impl Cursor {
    fn set_bidx(&mut self, bidx: usize, text: &str) {
        let (mut gidx, mut blen) = (0, 0);
        for g in text.graphemes(true) {
            let len = g.len();
            if len + blen > bidx {
                self.bidx = blen;
                self.gidx = gidx;
                return;
            }
            blen += len;
            if g == "\t" {
                gidx = (gidx / TAB_WIDTH) * TAB_WIDTH + TAB_WIDTH;
            } else {
                gidx += 1;
            }
        }
        self.bidx = blen;
        self.gidx = gidx;
    }

    fn reset(&mut self) {
        self.bidx = 0;
        self.gidx = 0;
    }
}
