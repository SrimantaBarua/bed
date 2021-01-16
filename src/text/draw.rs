// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::{Cell, RefCell};
use std::ops::Range;
use std::rc::Rc;

use euclid::{point2, size2, Point2D, Rect};

use crate::common::{split_text, PixelSize, RopeOrStr, SliceRange, SplitCbRes};
use crate::painter::WidgetCtx;
use crate::style::Color;
use crate::style::{StyleSubRanges, TextSize, TextStyle};

use super::{
    f26_6, FontCollectionHandle, FontCoreInner, GlyphAllocInfo, GlyphKey, ShapedSpan, ATLAS_SIZE,
};

pub(crate) const CURSOR_WIDTH: f32 = 2.0;

pub(crate) enum StyleType<'a, S: SliceRange> {
    Range(StyleSubRanges<'a, S>),
    Const(Range<usize>, TextStyle, Color, bool, f64),
}

impl<'a, S: SliceRange> StyleType<'a, S> {
    fn iter(self) -> StyleTypeIter<'a, S> {
        match self {
            StyleType::Range(r) => StyleTypeIter::Range(r),
            StyleType::Const(r, st, c, u, sc) => {
                StyleTypeIter::Const(Some((S::from_raw(r), st, c, u, sc)))
            }
        }
    }
}

enum StyleTypeIter<'a, S: SliceRange> {
    Range(StyleSubRanges<'a, S>),
    Const(Option<(S, TextStyle, Color, bool, f64)>),
}

impl<'a, S: SliceRange> Iterator for StyleTypeIter<'a, S> {
    type Item = (S, TextStyle, Color, bool, f64);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StyleTypeIter::Range(r) => r.next(),
            StyleTypeIter::Const(c) => c.take(),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum CursorStyle {
    Line,
    Block,
    Underline,
}

impl Default for CursorStyle {
    fn default() -> CursorStyle {
        CursorStyle::Block
    }
}

pub(crate) enum TextAlign {
    Left,
    Right,
}

// Cursor state for drawing
pub(crate) struct TextCursor {
    pub(crate) gidx: usize,
    pub(crate) style: CursorStyle,
    pub(crate) color: Color,
}

// Underline position and thickness
struct Underline {
    position: f32,
    thickness: f32,
    color: Color,
}

// Internal enum used when drawing line of text
enum SpanOrSpace {
    Span(ShapedSpan, Color, Option<Underline>),
    Space(f32, Option<Underline>),
}

// Use a font collection to render text
pub(crate) struct TextRenderCtx<'a, 'b> {
    pub(super) fc: &'a mut FontCollectionHandle,
    pub(super) core: Rc<RefCell<FontCoreInner>>,
    pub(super) ctx: &'a mut WidgetCtx<'b>,
}

impl<'a, 'b> TextRenderCtx<'a, 'b> {
    pub(crate) fn draw_line<S>(
        &mut self,
        line: &S,
        styles: StyleType<S::SliceRange>,
        tab_width: usize,
        origin: Point2D<f32, PixelSize>,
        width: f32,
        cursor: Option<TextCursor>,
        text_size: TextSize,
        align: TextAlign,
    ) -> (f32, f32)
    where
        S: RopeOrStr,
    {
        let mut style = TextStyle::default();
        let mut scale = 1.0;
        let mut space_metrics = self.fc.space_metrics(text_size, style);
        let mut font_metrics = self.fc.metrics(text_size);
        let mut sp_awidth = space_metrics.advance.width.to_f32();
        let mut ascender = space_metrics.ascender;
        let mut descender = space_metrics.descender;

        let gidx = Cell::new(0);
        let cursor_x = Cell::new(None);
        let current_x = Cell::new(origin.x);
        let cursor_underline_height = Cell::new(1.0);
        let cursor_underline_pos = Cell::new(-1.0);
        let cursor_block_width = Cell::new(sp_awidth);
        let fc = &mut self.fc;
        let spans = RefCell::new(Vec::new());

        for (range, cur_style, color, under, cur_scale) in styles.iter() {
            let text_size = text_size.scale(cur_scale);
            if cur_scale != scale {
                font_metrics = fc.metrics(text_size);
            }
            if cur_style != style || cur_scale != scale {
                space_metrics = fc.space_metrics(text_size, style);
                sp_awidth = space_metrics.advance.width.to_f32();
                style = cur_style;
                scale = cur_scale;
            }

            split_text(
                &line.slice_with(range),
                tab_width,
                |n| {
                    if let Some(cursor) = cursor.as_ref() {
                        if (gidx.get()..gidx.get() + n).contains(&cursor.gidx) {
                            cursor_x.set(Some(
                                current_x.get() + sp_awidth * (cursor.gidx - gidx.get()) as f32,
                            ));
                            cursor_block_width.set(sp_awidth);
                        }
                    }
                    gidx.set(gidx.get() + n);
                    current_x.set(current_x.get() + sp_awidth * n as f32);
                    let underline = if under {
                        Some(Underline {
                            position: font_metrics.underline_pos.to_f32(),
                            thickness: font_metrics.underline_thickness.to_f32(),
                            color,
                        })
                    } else {
                        None
                    };
                    spans
                        .borrow_mut()
                        .push(SpanOrSpace::Space(sp_awidth * (n as f32), underline));
                    if current_x.get() >= width {
                        SplitCbRes::Stop
                    } else {
                        SplitCbRes::Continue
                    }
                },
                |text| {
                    let shaped = fc.shape(text, text_size, style);
                    let mut gis = shaped.glyph_infos.iter().peekable();
                    for j in text.grapheme_idxs() {
                        while let Some(cluster) = gis.peek().map(|gi| gi.cluster) {
                            if cluster >= j as u32 {
                                break;
                            }
                            let gi = gis.next().unwrap();
                            current_x.set(current_x.get() + gi.advance.width.to_f32());
                        }
                        if let Some(cursor) = cursor.as_ref() {
                            if gidx.get() == cursor.gidx {
                                cursor_x.set(Some(current_x.get()));
                                cursor_underline_height.set(shaped.underline_thickness.to_f32());
                                cursor_underline_pos.set(shaped.underline_pos.to_f32());
                                if let Some(gi) = gis.peek() {
                                    cursor_block_width.set(gi.advance.width.to_f32());
                                }
                            }
                        }
                        gidx.set(gidx.get() + 1);
                    }
                    while let Some(gi) = gis.next() {
                        current_x.set(current_x.get() + gi.advance.width.to_f32());
                    }
                    if shaped.ascender > ascender {
                        ascender = shaped.ascender;
                    }
                    if shaped.descender > descender {
                        descender = shaped.descender;
                    }
                    let underline = if under {
                        Some(Underline {
                            position: shaped.underline_pos.to_f32(),
                            thickness: shaped.underline_thickness.to_f32(),
                            color,
                        })
                    } else {
                        None
                    };
                    spans
                        .borrow_mut()
                        .push(SpanOrSpace::Span(shaped, color, underline));
                    if current_x.get() >= width {
                        SplitCbRes::Stop
                    } else {
                        SplitCbRes::Continue
                    }
                },
            );
        }

        let text_width = current_x.get();
        let origin = if text_width >= width {
            origin
        } else {
            match align {
                TextAlign::Left => origin,
                TextAlign::Right => point2(origin.x + width - text_width, origin.y),
            }
        };

        if let Some(cursor) = cursor {
            let cursor_height = ascender - descender;
            let (cursor_width, cursor_height, cursor_y) = match cursor.style {
                CursorStyle::Line => (CURSOR_WIDTH, cursor_height.to_f32(), origin.y),
                CursorStyle::Block => (cursor_block_width.get(), cursor_height.to_f32(), origin.y),
                CursorStyle::Underline => (
                    cursor_block_width.get(),
                    cursor_underline_height.get() * 2.0,
                    origin.y + ascender.to_f32() - cursor_underline_pos.get(),
                ),
            };

            if let Some(x) = cursor_x.get() {
                let rect: Rect<f32, PixelSize> =
                    Rect::new(point2(x, cursor_y), size2(cursor_width, cursor_height));
                self.ctx.color_quad(rect, cursor.color, false);
            } else {
                let rect: Rect<f32, PixelSize> = Rect::new(
                    point2(current_x.get(), cursor_y),
                    size2(cursor_width, cursor_height),
                );
                self.ctx.color_quad(rect, cursor.color, false);
            }
        }

        let spans = spans.borrow();
        let mut pos = origin;
        pos.y += ascender.to_f32();
        for span_or_space in spans.iter() {
            if pos.x >= width {
                break;
            }
            match span_or_space {
                SpanOrSpace::Space(width, opt_under) => {
                    if let Some(under) = opt_under {
                        let rect = Rect::new(
                            point2(pos.x, pos.y - under.position),
                            size2(*width, under.thickness),
                        );
                        self.ctx.color_quad(rect, under.color, false);
                    }
                    pos.x += width;
                }
                SpanOrSpace::Span(shaped, color, opt_under) => {
                    self.draw(shaped, pos, *color);
                    if let Some(under) = opt_under {
                        let rect = Rect::new(
                            point2(pos.x, pos.y - under.position),
                            size2(shaped.width.to_f32(), under.thickness),
                        );
                        self.ctx.color_quad(rect, under.color, false);
                    }
                    pos.x += shaped.width.to_f32();
                }
            }
        }

        (ascender.to_f32(), descender.to_f32())
    }

    pub(crate) fn draw(
        &mut self,
        span: &ShapedSpan,
        origin: Point2D<f32, PixelSize>,
        color: Color,
    ) {
        let mut origin = point2(f26_6::from(origin.x), f26_6::from(origin.y));
        let core = &mut *self.core.borrow_mut();
        let font = core.id_font_map.get(&span.font_key).unwrap().clone();
        let font = &mut *font.borrow_mut();
        let font_key = font.num;
        let size = span.size;

        for gi in &span.glyph_infos {
            let base = origin + gi.offset;
            let base_floor = (base.x.floor(), base.y.floor());
            let base_offset = point2(base.x - base_floor.0, base.y - base_floor.1);
            let key = GlyphKey {
                font_key,
                size,
                glyph_id: gi.gid,
                origin: base_offset,
            };

            // FIXME: Optimize LRU glyph replacement algorithm. Now just drops entire cache (BAD)
            // Ideal procedure - remove LRU glyphs till there's space, allocate, rearrange
            // Rearranging involves copying glyphs to a second texture, then blitting that entire
            // texture to this one

            if !core.rastered_glyph_map.contains_key(&key) {
                if let Some(rastered) = font.raster.raster(base_offset, gi.gid, size) {
                    let atlas_alloc = &mut core.atlas_allocator;
                    loop {
                        match atlas_alloc.allocate(rastered.metrics.size.cast().to_untyped()) {
                            Some(allocation) => {
                                let glyph_rect = Rect::new(
                                    point2(allocation.rectangle.min.x, allocation.rectangle.min.y),
                                    rastered.metrics.size.cast(),
                                );
                                let atlas = self.ctx.text_atlas();
                                atlas.sub_image(glyph_rect.cast(), rastered.buffer);
                                let tex_rect = atlas.get_inverted_tex_dimension(glyph_rect);
                                let alloc_info = GlyphAllocInfo {
                                    tex_rect,
                                    metrics: rastered.metrics.clone(),
                                };
                                core.rastered_glyph_map
                                    .insert(key.clone(), Some(alloc_info));
                                break;
                            }
                            None => {
                                if core.rastered_glyph_map.len() == 0 {
                                    panic!("Glyph is too big!! Max size: {:?}", ATLAS_SIZE);
                                }
                                core.rastered_glyph_map.clear();
                                atlas_alloc.clear();
                            }
                        }
                    }
                } else {
                    core.rastered_glyph_map.insert(key.clone(), None);
                }
            }

            let opt_allocated = core.rastered_glyph_map.get_mut(&key).unwrap();
            if let Some(allocated) = opt_allocated {
                let rect_origin = point2(
                    base_floor.0.to_f32() + allocated.metrics.bearing.width as f32,
                    base_floor.1.to_f32() - allocated.metrics.bearing.height as f32,
                );
                let rect = Rect::new(rect_origin, allocated.metrics.size.cast());
                self.ctx.texture_color_quad(rect, allocated.tex_rect, color);
            }

            origin += gi.advance;
        }
    }
}
