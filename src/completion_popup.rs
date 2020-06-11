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
    dpi: Size2D<u32, DPI>,
    theme: Rc<Theme>,
    config: Rc<Config>,
    text_shaper: Rc<RefCell<TextShaper>>,
}

impl CompletionPopup {
    pub(crate) fn new(
        mut origin: Point2D<u32, PixelSize>,
        constrain_rect: Rect<u32, PixelSize>,
        options: Vec<CompletionOption>,
        theme: Rc<Theme>,
        config: Rc<Config>,
        text_shaper: Rc<RefCell<TextShaper>>,
        dpi: Size2D<u32, DPI>,
    ) -> Option<CompletionPopup> {
        let (ascender, descender) = {
            let shaper = &mut *text_shaper.borrow_mut();
            let raster = shaper
                .get_raster(config.completion_face, TextStyle::default())
                .unwrap();
            let metrics = raster.get_metrics(config.completion_font_size, dpi);
            (metrics.ascender, metrics.descender)
        };
        let height = (ascender - descender) as u32;

        if options.len() == 0 {
            return None;
        }
        let height_below = constrain_rect.size.height - (origin.y as i32 - descender) as u32;
        let height_above = origin.y - ascender as u32;
        let max_height = max(height_above, height_below);
        let mut shaped_lines = VecDeque::new();

        let (mut total_height, mut width) = (config.completion_padding_vertical * 2, 0);
        {
            let shaper = &mut *text_shaper.borrow_mut();
            for item in &options {
                if total_height + height > max_height {
                    break;
                }
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
                total_height += height;
                width = max(width, shaped.width() as u32);
                shaped_lines.push_back(shaped);
            }
        }
        if shaped_lines.len() == 0 {
            return None;
        }
        width += 2 * config.completion_padding_horizontal + MIDDLE_PADDING;

        if total_height > height_below {
            origin.y -= ascender as u32 + total_height;
        } else {
            origin.y = (origin.y as i32 - descender) as u32;
        }
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
            dpi,
            theme,
            config,
            text_shaper,
        })
    }

    pub(crate) fn draw(&self, painter: &mut Painter) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut painter = painter.widget_ctx(self.rect.cast(), self.theme.completion.background);
        let basex = self.config.completion_padding_horizontal as i32;
        let mut pos = point2(basex, self.config.completion_padding_vertical as i32);
        for line in &self.shaped {
            pos.y += self.ascender;
            painter.draw_shaped_text(
                shaper,
                pos,
                line,
                None,
                self.rect.size.width - (basex as u32) * 2,
            );
            pos.y -= self.descender;
            pos.x = basex;
        }
    }

    pub(crate) fn interacted(&self) -> bool {
        self.selected.is_some()
    }

    pub(crate) fn get_choice(&self) -> Option<&str> {
        self.selected.map(|i| self.options[i].option.as_ref())
    }
}
