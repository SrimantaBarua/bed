// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::rc::Rc;

use euclid::{point2, size2, Point2D, Rect, Size2D};

use crate::common::{PixelSize, DPI};
use crate::config::Config;
use crate::painter::Painter;
use crate::style::{Color, TextStyle};
use crate::text::{RopeOrStr, ShapedText, TextAlignment, TextShaper};
use crate::theme::Theme;

const MIDDLE_PADDING: u32 = 12;

pub(crate) struct CompletionOption {
    pub(crate) option: String,
    pub(crate) annotation: String,
    pub(crate) annotation_color: Color,
}

impl CompletionOption {
    pub(crate) fn new(
        option: String,
        annotation: String,
        annotation_color: Color,
    ) -> CompletionOption {
        CompletionOption {
            option,
            annotation,
            annotation_color,
        }
    }
}

pub(crate) struct CompletionPopup {
    options: Vec<CompletionOption>,
    shaped: VecDeque<ShapedText>,
    start: usize,
    selected: Option<usize>,
    rect: Rect<u32, PixelSize>,
    ascender: i32,
    descender: i32,
    height: u32,
    theme: Rc<Theme>,
    config: Rc<Config>,
    text_shaper: Rc<RefCell<TextShaper>>,
}

impl CompletionPopup {
    pub(crate) fn new(
        relative_origin: Point2D<u32, PixelSize>,
        constrain_rect: Rect<u32, PixelSize>,
        options: Vec<CompletionOption>,
        theme: Rc<Theme>,
        config: Rc<Config>,
        text_shaper: Rc<RefCell<TextShaper>>,
        dpi: Size2D<u32, DPI>,
        text_ascender: i32,
        text_descender: i32,
    ) -> Option<CompletionPopup> {
        let (ascender, descender) = {
            let shaper = &mut *text_shaper.borrow_mut();
            let raster = shaper
                .get_raster(config.completion_face, TextStyle::default())
                .unwrap();
            let metrics = raster.get_metrics(config.completion_font_size, dpi);
            (metrics.ascender, metrics.descender)
        };
        let height = (ascender - descender) as u32 + 2 * config.completion_line_padding;
        let mut origin = relative_origin;

        if options.len() == 0 {
            return None;
        }
        let height_below = constrain_rect.size.height - (origin.y as i32 - text_descender) as u32;
        let height_above = origin.y - text_ascender as u32;
        let max_height = max(height_above, height_below);
        let shaped_len = min(max_height / height, options.len() as u32);
        if shaped_len == 0 {
            return None;
        }
        let total_height = shaped_len * height + 2 * config.completion_padding_vertical;
        if total_height > height_below {
            origin.y -= text_ascender as u32 + total_height;
        } else {
            origin.y = (origin.y as i32 - text_descender) as u32;
        }

        let mut width = 0;
        let mut shaped_lines = VecDeque::new();
        {
            let shaper = &mut *text_shaper.borrow_mut();
            for item in &options {
                let olc = item.option.chars().count();
                let combined = item.option.clone() + &item.annotation;
                let rs = RopeOrStr::from(combined.as_ref());
                let lc = rs.len_chars();
                let shaped = shaper.shape_line(
                    rs,
                    dpi,
                    config.tab_width,
                    &[(lc, config.completion_face)],
                    &[(lc, TextStyle::default())],
                    &[
                        (olc, config.completion_font_size),
                        (lc, config.completion_font_size.scale(0.7)),
                    ],
                    &[
                        (olc, theme.completion.foreground),
                        (lc, item.annotation_color),
                    ],
                    &[(lc, None)],
                    &[(olc, TextAlignment::Left), (lc, TextAlignment::Right)],
                );
                width = max(width, shaped.width() as u32);
                shaped_lines.push_back(shaped);
            }
        }
        width += 2 * config.completion_padding_horizontal + MIDDLE_PADDING;
        width = min(width, constrain_rect.size.width);
        if origin.x + width > constrain_rect.size.width {
            origin.x = constrain_rect.size.width - width;
        }
        origin.x += constrain_rect.origin.x;
        origin.y += constrain_rect.origin.y;

        Some(CompletionPopup {
            options,
            start: 0,
            selected: None,
            shaped: shaped_lines,
            rect: Rect::new(origin, size2(width, total_height)),
            ascender,
            descender,
            height,
            theme,
            config,
            text_shaper,
        })
    }

    pub(crate) fn next(&mut self) {
        if let Some(idx) = self.selected {
            self.selected = Some((idx + 1) % self.options.len());
        } else {
            self.selected = Some(0);
        }
        let idx = self.selected.unwrap();
        if idx < self.start {
            self.start = idx;
        } else if idx >= self.start + self.visible_len() {
            self.start = idx - self.visible_len() + 1;
        }
    }

    pub(crate) fn prev(&mut self) {
        if let Some(idx) = self.selected {
            if idx == 0 {
                self.selected = Some(self.options.len() - 1);
            } else {
                self.selected = Some(idx - 1);
            }
        } else {
            self.selected = Some(self.options.len() - 1);
        }
        let idx = self.selected.unwrap();
        if idx < self.start {
            self.start = idx;
        } else if idx >= self.start + self.visible_len() {
            self.start = idx - self.visible_len() + 1;
        }
    }

    pub(crate) fn draw(&self, painter: &mut Painter) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut painter = painter.widget_ctx(self.rect.cast(), self.theme.completion.background);
        let basex = self.config.completion_padding_horizontal as i32;
        let mut pos = point2(basex, self.config.completion_padding_vertical as i32);

        for linum in self.start..(min(self.start + self.visible_len(), self.shaped.len())) {
            let line = &self.shaped[linum];
            if let Some(idx) = self.selected {
                if linum == idx {
                    painter.color_quad(
                        Rect::new(
                            point2(0, pos.y),
                            size2(self.rect.size.width, self.height).cast(),
                        ),
                        self.theme.completion.active_background,
                    );
                }
            }
            pos.y += self.ascender + self.config.completion_line_padding as i32;
            painter.draw_shaped_text(
                shaper,
                pos,
                line,
                None,
                self.rect.size.width - (basex as u32) * 2,
            );
            pos.y -= self.descender - self.config.completion_line_padding as i32;
            pos.x = basex;
        }
    }

    pub(crate) fn get_choice(&self) -> Option<String> {
        self.selected.map(|i| self.options[i].option.clone())
    }

    fn visible_len(&self) -> usize {
        (self.rect.size.height / self.height) as usize
    }
}
