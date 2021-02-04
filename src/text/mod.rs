// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::rc::Rc;

use euclid::{size2, Point2D, Rect, Size2D};
use fnv::{FnvBuildHasher, FnvHashMap};
use guillotiere::{AllocatorOptions, AtlasAllocator};
use lru_cache::LruCache;

use crate::common::{PixelSize, RopeOrStr, TextureSize};
use crate::painter::WidgetCtx;
use crate::style::{TextSize, TextStyle};

#[cfg(target_os = "windows")]
mod directwrite;
#[cfg(target_os = "linux")]
mod fontconfig;

#[cfg(target_os = "windows")]
use self::directwrite as font_source;
#[cfg(target_os = "linux")]
use self::fontconfig as font_source;

mod draw;
mod freetype;
mod harfbuzz;
mod types;

const SHAPED_SPAN_CAPACITY: usize = 4096;
const ATLAS_SIZE: Size2D<i32, PixelSize> = size2(4096, 4096);

use self::font_source::{Charset, FontSource, Pattern};
use self::freetype::{FontMetrics, GlyphMetrics, RasterCore, RasterFont};
use self::harfbuzz::{GlyphInfo, HbBuffer, HbFont};

pub(crate) use draw::{CursorStyle, StyleType, TextAlign, TextCursor, CURSOR_WIDTH};
pub(crate) use types::f26_6;

// Core handle for loading new fonts. Once fonts are loaded, this isn't used directly. However,
// this manages state that is shared by all fonts
pub(crate) struct FontCore(Rc<RefCell<FontCoreInner>>);

impl FontCore {
    pub(crate) fn new() -> FontCore {
        FontCore(Rc::new(RefCell::new(FontCoreInner::new())))
    }

    pub(crate) fn find(&mut self, family: &str) -> Option<FontCollectionHandle> {
        let family = self.0.borrow_mut().find(family)?;
        let collection = FontCollection::new(family, self.0.clone());
        Some(FontCollectionHandle(Rc::new(RefCell::new(collection))))
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
    atlas_allocator: AtlasAllocator,
    rastered_glyph_map: LruCache<GlyphKey, Option<GlyphAllocInfo>, FnvBuildHasher>,
    // Font stuff
    next_font_num: u16,
    path_font_map: FnvHashMap<(CString, u32), Rc<RefCell<FontFamily>>>,
    id_font_map: FnvHashMap<u16, Rc<RefCell<Font>>>,
    hb_buffer: HbBuffer,
    raster_core: RasterCore,
    font_source: FontSource,
}

impl FontCoreInner {
    fn new() -> FontCoreInner {
        let atlas_options = AllocatorOptions {
            alignment: size2(1, 1),
            small_size_threshold: 4,
            large_size_threshold: 256,
        };
        FontCoreInner {
            atlas_allocator: AtlasAllocator::with_options(ATLAS_SIZE.to_untyped(), &atlas_options),
            rastered_glyph_map: LruCache::with_hasher(4096 * 4096, FnvBuildHasher::default()),
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
}

// A handle to a font "collection". This is defined by a default font "family", and a list of
// fallback font families
#[derive(Clone)]
pub(crate) struct FontCollectionHandle(Rc<RefCell<FontCollection>>);

impl FontCollectionHandle {
    pub(crate) fn metrics(&mut self, size: TextSize) -> FontMetrics {
        self.0.borrow_mut().metrics(size)
    }

    pub(crate) fn space_metrics(&mut self, size: TextSize, style: TextStyle) -> GlyphMetrics {
        self.0.borrow_mut().space_metrics(size, style)
    }

    pub(crate) fn shape<S>(&mut self, text: &S, size: TextSize, style: TextStyle) -> ShapedSpan
    where
        S: RopeOrStr,
    {
        self.0.borrow_mut().shape(text, size, style)
    }

    pub(crate) fn render_ctx<'a, 'b>(
        &'a mut self,
        widget_ctx: &'a mut WidgetCtx<'b>,
    ) -> draw::TextRenderCtx<'a, 'b> {
        let core = self.0.borrow().core.clone();
        draw::TextRenderCtx {
            core,
            fc: self,
            ctx: widget_ctx,
        }
    }
}

struct FontCollection {
    families: Vec<Rc<RefCell<FontFamily>>>, // families[0] is the default family
    core: Rc<RefCell<FontCoreInner>>,
    cache: LruCache<(String, TextSize, TextStyle), ShapedSpan, FnvBuildHasher>,
}

impl FontCollection {
    fn metrics(&mut self, size: TextSize) -> FontMetrics {
        self.families[0].borrow_mut().fonts[&TextStyle::default()]
            .borrow_mut()
            .metrics(size)
    }

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
        font.space_metrics(size)
            .expect("failed to get space metrics")
    }

    fn shape<S>(&mut self, text: &S, size: TextSize, style: TextStyle) -> ShapedSpan
    where
        S: RopeOrStr,
    {
        assert!(text.blen() > 0);
        let text_string = text.string();
        // Check cache
        if let Some(shaped) = self.cache.get_mut(&(text_string.clone(), size, style)) {
            return shaped.clone();
        }
        // Otherwise, go the normal route
        let core = &mut *self.core.borrow_mut();
        let first_char = text.char_iter().next().unwrap();
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
        buffer.add_utf8(&text_string);
        buffer.guess_segment_properties();
        font.shaper.set_scale(size);
        let font_metrics = font.raster.get_metrics(size);
        let glyph_infos = harfbuzz::shape(&mut font.shaper, buffer).collect::<Vec<_>>();
        let width = glyph_infos
            .iter()
            .fold(f26_6::from(0.0), |width, gi| width + gi.advance.width);
        let ret = ShapedSpan {
            glyph_infos,
            size,
            font_key: font.num,
            ascender: font_metrics.ascender,
            descender: font_metrics.descender,
            underline_pos: font_metrics.underline_pos,
            underline_thickness: font_metrics.underline_thickness,
            width,
        };
        self.cache.insert((text_string, size, style), ret.clone());
        ret
    }

    fn new(family: Rc<RefCell<FontFamily>>, core: Rc<RefCell<FontCoreInner>>) -> FontCollection {
        FontCollection {
            families: vec![family],
            core,
            cache: LruCache::with_hasher(SHAPED_SPAN_CAPACITY, FnvBuildHasher::default()),
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
    metrics: FnvHashMap<TextSize, FontMetrics>,
    space_metrics: FnvHashMap<TextSize, GlyphMetrics>,
}

impl Font {
    fn new(core: &mut RasterCore, path: &CStr, idx: u32, num: u16) -> Option<Font> {
        let raster = core.new_face(path, idx)?;
        let shaper = HbFont::new(path, idx)?;
        Some(Font {
            raster,
            shaper,
            num,
            metrics: FnvHashMap::default(),
            space_metrics: FnvHashMap::default(),
        })
    }

    fn metrics(&mut self, size: TextSize) -> FontMetrics {
        if let Some(metrics) = self.metrics.get(&size) {
            metrics.clone()
        } else {
            let metrics = self.raster.get_metrics(size);
            self.metrics.insert(size, metrics.clone());
            metrics
        }
    }

    fn space_metrics(&mut self, size: TextSize) -> Option<GlyphMetrics> {
        if let Some(metrics) = self.space_metrics.get(&size) {
            Some(metrics.clone())
        } else {
            let gid = self.raster.get_glyph_for_char(' ');
            if gid == 0 {
                None
            } else {
                if let Some(metrics) = self.raster.get_glyph_metrics(gid, size) {
                    self.space_metrics.insert(size, metrics.clone());
                    Some(metrics)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct ShapedSpan {
    pub(crate) ascender: f26_6,
    pub(crate) descender: f26_6,
    pub(crate) underline_pos: f26_6,
    pub(crate) underline_thickness: f26_6,
    pub(crate) width: f26_6,
    pub(crate) glyph_infos: Vec<GlyphInfo>,
    size: TextSize,
    font_key: u16,
}
