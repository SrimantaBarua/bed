// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::cmp::{max, min};
use std::fmt::Write;
use std::rc::Rc;

use euclid::{point2, size2, Point2D, Rect, Size2D};

use crate::common::{PixelSize, DPI};
use crate::config::Config;
use crate::language_client::{
    DiagnosticCode, DiagnosticSeverity, HoverContents, MarkedString, MarkupKind,
};
use crate::painter::Painter;
use crate::style::TextStyle;
use crate::text::{RopeOrStr, ShapedText, TextAlignment, TextShaper};
use crate::theme::Theme;

pub(crate) struct HoverPopup {
    contents: Vec<(usize, ShapedText)>,
    diagnostic: Vec<(usize, ShapedText)>,
    rect: Rect<u32, PixelSize>,
    ascender: i32,
    descender: i32,
    height: u32,
    theme: Rc<Theme>,
    config: Rc<Config>,
    dpi: Size2D<u32, DPI>,
    text_shaper: Rc<RefCell<TextShaper>>,
    bound_width: u32,
    text_ascender: i32,
    text_descender: i32,
    original_origin: Point2D<u32, PixelSize>,
    constrain_rect: Rect<u32, PixelSize>,
}

impl HoverPopup {
    pub(crate) fn empty(
        relative_origin: Point2D<u32, PixelSize>,
        constrain_rect: Rect<u32, PixelSize>,
        theme: Rc<Theme>,
        config: Rc<Config>,
        text_shaper: Rc<RefCell<TextShaper>>,
        dpi: Size2D<u32, DPI>,
        text_ascender: i32,
        text_descender: i32,
    ) -> HoverPopup {
        let (ascender, descender) = {
            let shaper = &mut *text_shaper.borrow_mut();
            let raster = shaper
                .get_raster(config.hover_face, TextStyle::default())
                .unwrap();
            let metrics = raster.get_metrics(config.hover_font_size, dpi);
            (metrics.ascender, metrics.descender)
        };
        let height = (ascender - descender) as u32 + 2 * config.hover_line_padding;
        let height_above = relative_origin.y - text_ascender as u32;

        let bound_width = constrain_rect.size.width - 2 * config.hover_padding_horizontal;
        let total_height = 2 * config.hover_padding_vertical;
        let width = min(
            2 * config.hover_padding_horizontal,
            constrain_rect.size.width,
        );

        let mut origin = relative_origin;
        origin.x += constrain_rect.origin.x;
        origin.y += constrain_rect.origin.y;

        HoverPopup {
            contents: Vec::new(),
            diagnostic: Vec::new(),
            height,
            rect: Rect::new(origin, size2(width, total_height)),
            dpi,
            ascender,
            descender,
            config,
            theme,
            text_shaper,
            bound_width,
            text_ascender,
            text_descender,
            original_origin: relative_origin,
            constrain_rect,
        }
    }

    pub(crate) fn with_diagnostics(
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
        let original_origin = origin;
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
        origin.x += constrain_rect.origin.x;
        origin.y += constrain_rect.origin.y;

        Some(HoverPopup {
            contents: Vec::new(),
            diagnostic: shaped_lines,
            height,
            rect: Rect::new(origin, size2(width, total_height)),
            dpi,
            ascender,
            descender,
            config,
            theme,
            text_shaper,
            bound_width,
            text_ascender,
            text_descender,
            original_origin,
            constrain_rect,
        })
    }

    pub(crate) fn update_contents(&mut self, hover: HoverContents) {
        let config = self.config.clone();
        let theme = self.theme.clone();
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut total_height = self.rect.size.height + self.config.hover_padding_vertical;
        let mut width = self.rect.size.width - 2 * self.config.hover_padding_horizontal;
        let dpi = self.dpi;
        let bound_width = self.bound_width;
        let contents = &mut self.contents;
        let height_above = self.original_origin.y - self.text_ascender as u32;

        if contents.len() > 0 || self.diagnostic.len() == 0 {
            total_height -= self.config.hover_padding_vertical;
        }
        contents.clear();

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
            let height = shaped.metrics.height() as u32 + 2 * config.hover_line_padding;
            total_height += height;
            let mut num_lines = 1;
            if shaped_width <= bound_width {
                width = max(width, shaped_width);
            } else {
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
            contents.push((num_lines, shaped));
        };

        match hover {
            HoverContents::Str(string) => match string {
                MarkedString::Str(string) => {
                    for line in string.split('\n').map(|line| line.trim()) {
                        let rs = RopeOrStr::from(line);
                        shape_line(rs);
                    }
                }
                MarkedString::Code { value, .. } => {
                    for line in value.split('\n').map(|line| line.trim()) {
                        let rs = RopeOrStr::from(line);
                        shape_line(rs);
                    }
                }
            },
            HoverContents::Strings(strings) => {
                for string in strings {
                    match string {
                        MarkedString::Str(string) => {
                            for line in string.split('\n').map(|line| line.trim()) {
                                let rs = RopeOrStr::from(line);
                                shape_line(rs);
                            }
                        }
                        MarkedString::Code { value, .. } => {
                            for line in value.split('\n').map(|line| line.trim()) {
                                let rs = RopeOrStr::from(line);
                                shape_line(rs);
                            }
                        }
                    }
                }
            }
            HoverContents::Content(markup) => match markup.kind {
                MarkupKind::PlainText => {
                    for line in markup.value.split('\n').map(|line| line.trim()) {
                        let rs = RopeOrStr::from(line);
                        shape_line(rs);
                    }
                }
                MarkupKind::Markdown => {
                    for line in markup.value.split('\n').map(|line| line.trim()) {
                        let rs = RopeOrStr::from(line);
                        shape_line(rs);
                    }
                }
            },
        }
        if contents.len() == 0 {
            return;
        }

        width += 2 * config.hover_padding_horizontal;
        width = min(width, self.constrain_rect.size.width);

        let mut origin = self.original_origin;
        if total_height <= height_above {
            origin.y -= self.text_ascender as u32 + total_height;
        } else {
            origin.y = (origin.y as i32 - self.text_descender) as u32;
        }
        if origin.x + width > self.constrain_rect.size.width {
            origin.x = self.constrain_rect.size.width - width;
        }
        origin.x += self.constrain_rect.origin.x;
        origin.y += self.constrain_rect.origin.y;

        self.rect = Rect::new(origin, size2(width, total_height));
    }

    pub(crate) fn draw(&self, painter: &mut Painter) {
        if self.contents.len() == 0 && self.diagnostic.len() == 0 {
            return;
        }

        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut painter = painter.widget_ctx(self.rect.cast(), self.theme.hover.background, true);
        let basex = self.config.hover_padding_horizontal as i32;
        let width = self.rect.size.width - basex as u32;
        let mut pos = point2(basex, self.config.hover_padding_vertical as i32);

        if self.contents.len() > 0 {
            for (num_lines, line) in &self.contents {
                let height = line.metrics.height() as u32;
                pos.y += line.metrics.ascender + self.config.hover_line_padding as i32;
                painter.draw_shaped_text(shaper, pos, line, None, width, height, true);
                pos.y -= line.metrics.descender - self.config.hover_line_padding as i32;
                pos.y += (num_lines - 1) as i32 * height as i32;
                pos.x = basex;
            }
            pos.y += self.config.hover_padding_vertical as i32;
        }

        for (num_lines, line) in &self.diagnostic {
            pos.y += self.ascender + self.config.hover_line_padding as i32;
            painter.draw_shaped_text(shaper, pos, line, None, width, self.height, true);
            pos.y -= self.descender - self.config.hover_line_padding as i32;
            pos.y += (num_lines - 1) as i32 * self.height as i32;
            pos.x = basex;
        }
    }
}
