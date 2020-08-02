// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::rc::Rc;

use euclid::{point2, size2, Point2D, Rect, Size2D};
use fnv::FnvHashMap;
use guillotiere::{AllocatorOptions, AtlasAllocator};

use crate::common::{PixelSize, TextureSize};
use crate::opengl::{ElemArr, GlTexture, Mat4, ShaderProgram, TexRed, TexUnit};
use crate::shapes::TexColorQuad;
use crate::style::{Color, TextSize, TextStyle};

#[cfg(target_os = "linux")]
mod fontconfig;

#[cfg(target_os = "linux")]
use self::fontconfig as font_source;

mod freetype;
mod harfbuzz;
mod types;

use self::font_source::{Charset, FontSource, Pattern};
use self::freetype::{GlyphMetrics, RasterCore, RasterFont};
use self::harfbuzz::{GlyphInfo, HbBuffer, HbFont};

pub(crate) use types::f26_6;

// Core handle for loading new fonts. Once fonts are loaded, this isn't used directly. However,
// this manages state that is shared by all fonts
pub(crate) struct FontCore(Rc<RefCell<FontCoreInner>>);

impl FontCore {
    pub(crate) fn new(window_size: Size2D<f32, PixelSize>) -> FontCore {
        FontCore(Rc::new(RefCell::new(FontCoreInner::new(window_size))))
    }

    pub(crate) fn find(&mut self, family: &str) -> Option<FontCollectionHandle> {
        let family = {
            let inner = &mut *self.0.borrow_mut();
            inner.find(family)?
        };
        let collection = FontCollection::new(family, self.0.clone());
        Some(FontCollectionHandle(Rc::new(RefCell::new(collection))))
    }

    pub(crate) fn set_window_size(&mut self, size: Size2D<f32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        inner.set_window_size(size);
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct GlyphKey {
    font_key: u16,
    size: TextSize,
    glyph_id: u32,
    origin: Point2D<f26_6, PixelSize>,
}

struct GlyphAllocInfo {
    tex_rect: Rect<f32, TextureSize>,
    metrics: GlyphMetrics,
}

struct FontCoreInner {
    // OpenGL stuff
    shader: ShaderProgram,
    quad_arr: ElemArr<TexColorQuad>,
    atlas: GlTexture<TexRed>,
    atlas_allocator: AtlasAllocator,
    rastered_glyph_map: FnvHashMap<GlyphKey, Option<GlyphAllocInfo>>,
    // Font stuff
    next_font_num: u16,
    path_font_map: FnvHashMap<(CString, u32), Rc<RefCell<FontFamily>>>,
    id_font_map: FnvHashMap<u16, Rc<RefCell<Font>>>,
    hb_buffer: HbBuffer,
    raster_core: RasterCore,
    font_source: FontSource,
}

impl FontCoreInner {
    fn new(window_size: Size2D<f32, PixelSize>) -> FontCoreInner {
        let vsrc = include_str!("shader_src/tex_color_quad.vert");
        let fsrc = include_str!("shader_src/tex_color_quad.frag");
        let mut shader = ShaderProgram::new(vsrc, fsrc).unwrap();
        let projection = Mat4::projection(window_size);
        {
            let mut active = shader.use_program();
            active.uniform_mat4f(
                CStr::from_bytes_with_nul(b"projection\0").unwrap(),
                &projection,
            );
        }
        let atlas_size = size2(4096, 4096);
        let atlas_options = AllocatorOptions {
            snap_size: 1,
            small_size_threshold: 4,
            large_size_threshold: 256,
        };
        FontCoreInner {
            shader,
            quad_arr: ElemArr::new(4096),
            atlas: GlTexture::new(TexUnit::Texture0, atlas_size),
            atlas_allocator: AtlasAllocator::with_options(
                atlas_size.cast().to_untyped(),
                &atlas_options,
            ),
            rastered_glyph_map: FnvHashMap::default(),
            next_font_num: 0,
            path_font_map: FnvHashMap::default(),
            id_font_map: FnvHashMap::default(),
            hb_buffer: HbBuffer::new(),
            raster_core: RasterCore::new(),
            font_source: FontSource::new(),
        }
    }

    fn find(&mut self, family: &str) -> Option<Rc<RefCell<FontFamily>>> {
        let default_style = TextStyle::default();
        let mut pattern = Pattern::new();
        pattern.set_family(family);
        pattern.set_slant(default_style.slant);
        pattern.set_weight(default_style.weight);
        let (family, path, idx) = self.font_source.find_match(&mut pattern)?;
        if let Some(family) = self.path_font_map.get(&(path.clone(), idx)) {
            Some(family.clone())
        } else {
            let font = Font::new(&mut self.raster_core, &path, idx, self.next_font_num)?;
            let font = Rc::new(RefCell::new(font));
            let family = Rc::new(RefCell::new(FontFamily::new(family, font.clone())));
            self.path_font_map.insert((path, idx), family.clone());
            self.id_font_map.insert(self.next_font_num, font);
            self.next_font_num += 1;
            Some(family)
        }
    }

    fn flush_glyphs(&mut self) {
        let active = self.shader.use_program();
        self.quad_arr.flush(&active);
    }

    fn set_window_size(&mut self, size: Size2D<f32, PixelSize>) {
        let projection = Mat4::projection(size);
        {
            let mut active = self.shader.use_program();
            active.uniform_mat4f(
                CStr::from_bytes_with_nul(b"projection\0").unwrap(),
                &projection,
            );
        }
    }
}

// A handle to a font "collection". This is defined by a default font "family", and a list of
// fallback font families
#[derive(Clone)]
pub(crate) struct FontCollectionHandle(Rc<RefCell<FontCollection>>);

impl FontCollectionHandle {
    pub(crate) fn space_metrics(&mut self, size: TextSize, style: TextStyle) -> GlyphMetrics {
        let inner = &mut *self.0.borrow_mut();
        inner.space_metrics(size, style)
    }
    pub(crate) fn shape(&mut self, text: &str, size: TextSize, style: TextStyle) -> ShapedSpan {
        let inner = &mut *self.0.borrow_mut();
        inner.shape(text, size, style)
    }

    pub(crate) fn flush_glyphs(&mut self) {
        let inner = &mut *self.0.borrow_mut();
        inner.flush_glyphs();
    }
}

struct FontCollection {
    families: Vec<Rc<RefCell<FontFamily>>>, // families[0] is the default family
    core: Rc<RefCell<FontCoreInner>>,
}

impl FontCollection {
    fn space_metrics(&mut self, size: TextSize, style: TextStyle) -> GlyphMetrics {
        let core = &mut *self.core.borrow_mut();
        let family = &mut *self.families[0].borrow_mut();
        let font = family.get(style).unwrap_or_else(|| {
            let mut pattern = Pattern::new();
            pattern.set_family(&family.family);
            pattern.set_slant(style.slant);
            pattern.set_weight(style.weight);
            core.font_source
                .find_match(&mut pattern)
                .and_then(|(_, path, idx)| {
                    let font = Font::new(&mut core.raster_core, &path, idx, core.next_font_num)?;
                    let font = Rc::new(RefCell::new(font));
                    family.set(style, font.clone());
                    core.id_font_map.insert(core.next_font_num, font.clone());
                    core.next_font_num += 1;
                    Some(font)
                })
                .unwrap_or_else(|| family.get(TextStyle::default()).unwrap())
        });
        let font = &mut *font.borrow_mut();
        let gid = font.raster.get_glyph_for_char(' ');
        assert!(gid != 0, "Failed to get glyph for space");
        font.raster
            .get_glyph_metrics(gid, size)
            .expect("Failed to get glyph metrics for space")
    }

    fn shape(&mut self, text: &str, size: TextSize, style: TextStyle) -> ShapedSpan {
        assert!(text.len() > 0);
        let ret_core = self.core.clone();
        let core = &mut *self.core.borrow_mut();
        let first_char = text.chars().next().unwrap();
        let families = &mut self.families;
        let family_idx = families
            .iter()
            .position(|family| {
                let family = &*family.borrow();
                let font = family.get(TextStyle::default()).unwrap();
                let font = &*font.borrow();
                font.raster.has_glyph_for_char(first_char)
            })
            .or_else(|| {
                let mut charset = Charset::new();
                charset.add_char(first_char);
                let mut pattern = Pattern::new();
                pattern.set_slant(style.slant);
                pattern.set_weight(style.weight);
                pattern.add_charset(charset);
                core.font_source
                    .find_match(&mut pattern)
                    .and_then(|(family, path, idx)| {
                        if let Some(font) =
                            Font::new(&mut core.raster_core, &path, idx, core.next_font_num)
                        {
                            let font = Rc::new(RefCell::new(font));
                            let family =
                                Rc::new(RefCell::new(FontFamily::new(family, font.clone())));
                            core.path_font_map.insert((path, idx), family.clone());
                            core.id_font_map.insert(core.next_font_num, font);
                            core.next_font_num += 1;
                            families.push(family);
                            Some(families.len() - 1)
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or(0);
        let family = &mut *self.families[family_idx].borrow_mut();
        let font = family.get(style).unwrap_or_else(|| {
            let mut pattern = Pattern::new();
            pattern.set_family(&family.family);
            pattern.set_slant(style.slant);
            pattern.set_weight(style.weight);
            core.font_source
                .find_match(&mut pattern)
                .and_then(|(_, path, idx)| {
                    let font = Font::new(&mut core.raster_core, &path, idx, core.next_font_num)?;
                    let font = Rc::new(RefCell::new(font));
                    family.set(style, font.clone());
                    core.id_font_map.insert(core.next_font_num, font.clone());
                    core.next_font_num += 1;
                    Some(font)
                })
                .unwrap_or_else(|| family.get(TextStyle::default()).unwrap())
        });
        let font = &mut *font.borrow_mut();
        let buffer = &mut core.hb_buffer;
        buffer.clear_contents();
        buffer.add_utf8(text);
        buffer.guess_segment_properties();
        font.shaper.set_scale(size);
        let font_metrics = font.raster.get_metrics(size);
        let glyph_infos = harfbuzz::shape(&mut font.shaper, buffer).collect::<Vec<_>>();
        let width = glyph_infos
            .iter()
            .fold(f26_6::from(0.0), |width, gi| width + gi.advance.width);
        ShapedSpan {
            glyph_infos,
            size,
            font_key: font.num,
            ascender: font_metrics.ascender,
            descender: font_metrics.descender,
            underline_pos: font_metrics.underline_pos,
            underline_thickness: font_metrics.underline_thickness,
            width,
            core: ret_core,
        }
    }

    fn flush_glyphs(&mut self) {
        let core = &mut *self.core.borrow_mut();
        core.flush_glyphs();
    }

    fn new(family: Rc<RefCell<FontFamily>>, core: Rc<RefCell<FontCoreInner>>) -> FontCollection {
        FontCollection {
            families: vec![family],
            core,
        }
    }
}

struct FontFamily {
    family: String,
    fonts: FnvHashMap<TextStyle, Rc<RefCell<Font>>>,
}

impl FontFamily {
    fn new(family: String, font: Rc<RefCell<Font>>) -> FontFamily {
        let mut fonts = FnvHashMap::default();
        fonts.insert(TextStyle::default(), font);
        FontFamily { family, fonts }
    }

    fn get(&self, style: TextStyle) -> Option<Rc<RefCell<Font>>> {
        self.fonts.get(&style).map(|font| font.clone())
    }

    fn set(&mut self, style: TextStyle, font: Rc<RefCell<Font>>) {
        self.fonts.insert(style, font);
    }
}

struct Font {
    num: u16,
    raster: RasterFont,
    shaper: HbFont,
}

impl Font {
    fn new(core: &mut RasterCore, path: &CStr, idx: u32, num: u16) -> Option<Font> {
        let raster = core.new_face(path, idx)?;
        let shaper = HbFont::new(path, idx)?;
        Some(Font {
            raster,
            shaper,
            num,
        })
    }
}

pub(crate) struct ShapedSpan {
    pub(crate) ascender: f26_6,
    pub(crate) descender: f26_6,
    pub(crate) underline_pos: f26_6,
    pub(crate) underline_thickness: f26_6,
    pub(crate) width: f26_6,
    pub(crate) glyph_infos: Vec<GlyphInfo>,
    size: TextSize,
    font_key: u16,
    core: Rc<RefCell<FontCoreInner>>,
}

impl ShapedSpan {
    pub(crate) fn draw(&self, origin: Point2D<f32, PixelSize>, color: Color) {
        let mut origin = point2(f26_6::from(origin.x), f26_6::from(origin.y));
        let core = &mut *self.core.borrow_mut();
        let font = core.id_font_map.get(&self.font_key).unwrap().clone();
        let font = &mut *font.borrow_mut();
        let font_key = font.num;
        let size = self.size;

        for gi in &self.glyph_infos {
            let base = origin + gi.offset;
            let base_floor = (base.x.floor(), base.y.floor());
            let base_offset = point2(base.x - base_floor.0, base.y - base_floor.1);
            let key = GlyphKey {
                font_key,
                size,
                glyph_id: gi.gid,
                origin: base_offset,
            };

            if !core.rastered_glyph_map.contains_key(&key) {
                if let Some(rastered) = font.raster.raster(base_offset, gi.gid, size) {
                    loop {
                        if let Some(allocation) = core
                            .atlas_allocator
                            .allocate(rastered.metrics.size.cast().to_untyped())
                        {
                            let glyph_rect = Rect::new(
                                point2(allocation.rectangle.min.x, allocation.rectangle.min.y),
                                rastered.metrics.size.cast(),
                            );
                            core.atlas.sub_image(glyph_rect.cast(), rastered.buffer);
                            let tex_rect = core.atlas.get_inverted_tex_dimension(glyph_rect);
                            let allocation = GlyphAllocInfo {
                                tex_rect,
                                metrics: rastered.metrics.clone(),
                            };
                            core.rastered_glyph_map
                                .insert(key.clone(), Some(allocation));

                            /*
                            // Print allocation info
                            let mut total_size = 4096.0 * 4096.0;
                            let mut free_size = 0;
                            core.atlas_allocator.for_each_free_rectangle(|rect| {
                                free_size += rect.area();
                            });
                            eprintln!("Free space in atlas: {}%", free_size as f64 * 100.0 / total_size);
                            */

                            break;
                        }
                        /* TODO: Clear LRU glyph info */
                        /* TODO: Rearrange to reduce fragmentation */
                        /* TODO: Flush existing glyphs if any glyphs involved in current draw call
                         * are overwritten*/
                        unimplemented!();
                    }
                } else {
                    core.rastered_glyph_map.insert(key.clone(), None);
                }
            }

            let opt_allocated = core.rastered_glyph_map.get(&key).unwrap();
            if let Some(allocated) = opt_allocated {
                let rect_origin = point2(
                    base_floor.0.to_f32() + allocated.metrics.bearing.width as f32,
                    base_floor.1.to_f32() - allocated.metrics.bearing.height as f32,
                );
                let rect = Rect::new(rect_origin, allocated.metrics.size.cast());
                let tex_quad = TexColorQuad::new(rect, allocated.tex_rect, color);
                core.quad_arr.push(tex_quad);
                origin += gi.advance;
            }
        }
    }
}
