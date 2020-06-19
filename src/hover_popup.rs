// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::cmp::{max, min};
use std::fmt::Write;
use std::rc::Rc;

use euclid::{point2, size2, Point2D, Rect, Size2D};

use crate::common::{PixelSize, DPI};
use crate::config::Config;
use crate::language_client::{DiagnosticCode, DiagnosticSeverity};
use crate::painter::Painter;
use crate::style::TextStyle;
use crate::text::{RopeOrStr, ShapedText, TextAlignment, TextShaper};
use crate::theme::Theme;

pub(crate) struct HoverPopup {
    diagnostic: Vec<(usize, ShapedText)>,
    rect: Rect<u32, PixelSize>,
    ascender: i32,
    descender: i32,
    height: u32,
    theme: Rc<Theme>,
    config: Rc<Config>,
    text_shaper: Rc<RefCell<TextShaper>>,
}

impl HoverPopup {
    pub(crate) fn new(
        relative_origin: Point2D<u32, PixelSize>,
        constrain_rect: Rect<u32, PixelSize>,
        severity: &DiagnosticSeverity,
        code: Option<&DiagnosticCode>,
        source: Option<&str>,
        message: &str,
        theme: Rc<Theme>,
        config: Rc<Config>,
        text_shaper: Rc<RefCell<TextShaper>>,
        dpi: Size2D<u32, DPI>,
        text_ascender: i32,
        text_descender: i32,
    ) -> Option<HoverPopup> {
        let (ascender, descender) = {
            let shaper = &mut *text_shaper.borrow_mut();
            let raster = shaper
                .get_raster(config.hover_face, TextStyle::default())
                .unwrap();
            let metrics = raster.get_metrics(config.hover_font_size, dpi);
            (metrics.ascender, metrics.descender)
        };
        let height = (ascender - descender) as u32 + 2 * config.hover_line_padding;
        let mut origin = relative_origin;
        let height_above = origin.y - text_ascender as u32;

        let bound_width = constrain_rect.size.width - 2 * config.hover_padding_horizontal;
        let (mut total_height, mut width) = (2 * config.hover_padding_vertical, 0);
        let mut shaped_lines = Vec::new();
        {
            let shaper = &mut *text_shaper.borrow_mut();

            let mut shape_line = |rs: RopeOrStr| {
                let lc = rs.len_chars();
                let shaped = shaper.shape_line(
                    rs,
                    dpi,
                    config.tab_width,
                    &[(lc, config.hover_face)],
                    &[(lc, TextStyle::default())],
                    &[(lc, config.hover_font_size)],
                    &[(lc, theme.hover.foreground)],
                    &[(lc, None)],
                    &[(lc, TextAlignment::Left)],
                );
                let shaped_width = shaped.width() as u32;
                total_height += height;
                let mut num_lines = 1;
                if shaped_width <= bound_width {
                    width = max(width, shaped_width);
                } else {
                    width = bound_width;
                    let mut line_width = 0;
                    for (clusters, _, _, _, _, _, _) in shaped.styled_iter() {
                        let chunk_width = clusters.width() as u32;
                        if line_width > 0 && chunk_width + line_width > bound_width {
                            line_width = 0;
                            total_height += height;
                            num_lines += 1;
                        } else {
                            line_width += chunk_width;
                            width = max(width, line_width);
                        }
                    }
                }
                shaped_lines.push((num_lines, shaped));
            };

            // Shape message
            for line in message.split('\n').map(|line| line.trim_end()) {
                let rs = RopeOrStr::from(line);
                shape_line(rs);
            }
            // Shape source/code if required
            if let Some(source) = source {
                let mut source_line = source.to_owned();
                if let Some(code) = code {
                    write!(&mut source_line, "({})", code).unwrap();
                }
                let rs = RopeOrStr::from(source_line.as_ref());
                shape_line(rs);
            }
        }
        if shaped_lines.len() == 0 {
            return None;
        }

        width += 2 * config.hover_padding_horizontal;
        width = min(width, constrain_rect.size.width);

        if total_height <= height_above {
            origin.y -= text_ascender as u32 + total_height;
        } else {
            origin.y = (origin.y as i32 - text_descender) as u32;
        }
        if origin.x + width > constrain_rect.size.width {
            origin.x = constrain_rect.size.width - width;
        }

        Some(HoverPopup {
            diagnostic: shaped_lines,
            height,
            rect: Rect::new(origin, size2(width, total_height)),
            ascender,
            descender,
            config,
            theme,
            text_shaper,
        })
    }

    pub(crate) fn draw(&self, painter: &mut Painter) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut painter = painter.widget_ctx(self.rect.cast(), self.theme.hover.background, true);
        let basex = self.config.hover_padding_horizontal as i32;
        let width = self.rect.size.width - basex as u32;
        let mut pos = point2(basex, self.config.hover_padding_vertical as i32);
        for (num_lines, line) in &self.diagnostic {
            pos.y += self.ascender + self.config.hover_line_padding as i32;
            painter.draw_shaped_text(shaper, pos, line, None, width, self.height, true);
            pos.y -= self.descender - self.config.hover_line_padding as i32;
            pos.y += (num_lines - 1) as i32 * self.height as i32;
            pos.x = basex;
        }
    }
}
