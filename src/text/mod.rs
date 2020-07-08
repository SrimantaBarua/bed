// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::rc::Rc;

use fnv::FnvHashMap;

use crate::style::TextStyle;

#[cfg(target_os = "linux")]
mod fontconfig;

#[cfg(target_os = "linux")]
use self::fontconfig as font_source;

mod freetype;
mod harfbuzz;

use self::freetype::{RasterCore, RasterFace};
use self::harfbuzz::{HbBuffer, HbFont};

// Core handle for loading new fonts. Once fonts are loaded, this isn't used directly. However,
// this manages state that is shared by all fonts
pub(crate) struct FontCore(Rc<RefCell<FontCoreInner>>);

impl FontCore {
    pub(crate) fn new() -> FontCore {
        FontCore(Rc::new(RefCell::new(FontCoreInner::new())))
    }

    pub(crate) fn find(&mut self, family: &str) -> Option<FontCollection> {
        let coreref = self.0.clone();
        let inner = &mut *self.0.borrow_mut();
        inner.find(family, coreref)
    }
}

struct FontCoreInner {
    path_font_map: FnvHashMap<(CString, u32), Rc<RefCell<FontFamily>>>,
    raster_core: RasterCore,
    hb_buffer: HbBuffer,
    source: font_source::FontSource,
}

impl FontCoreInner {
    fn new() -> FontCoreInner {
        FontCoreInner {
            source: font_source::FontSource::new(),
            raster_core: RasterCore::new(),
            hb_buffer: HbBuffer::new(),
            path_font_map: FnvHashMap::default(),
        }
    }

    fn find(&mut self, family: &str, core: Rc<RefCell<FontCoreInner>>) -> Option<FontCollection> {
        let default_style = TextStyle::default();
        let mut pattern = font_source::Pattern::new()?;
        if !pattern.set_family(family)
            || !pattern.set_slant(default_style.slant)
            || !pattern.set_weight(default_style.weight)
        {
            return None;
        }
        let (family, path, idx) = self.source.find_match(&mut pattern)?;
        if let Some(family) = self.path_font_map.get(&(path.clone(), idx)) {
            Some(FontCollection::new(family.clone(), core))
        } else {
            let family = FontFamily::new(&mut self.raster_core, family, &path, idx)?;
            let family = Rc::new(RefCell::new(family));
            self.path_font_map.insert((path, idx), family.clone());
            Some(FontCollection::new(family.clone(), core))
        }
    }
}

// A handle to a font "collection". This is defined by a default font "family", and a list of
// fallback font families
pub(crate) struct FontCollection {
    families: Vec<Rc<RefCell<FontFamily>>>, // families[0] is the default family
    core: Rc<RefCell<FontCoreInner>>,
}

impl FontCollection {
    fn new(family: Rc<RefCell<FontFamily>>, core: Rc<RefCell<FontCoreInner>>) -> FontCollection {
        FontCollection {
            families: vec![family],
            core,
        }
    }
}

struct FontFamily {
    family: String,
    fonts: FnvHashMap<TextStyle, Font>,
}

impl FontFamily {
    fn new(core: &mut RasterCore, family: String, path: &CStr, idx: u32) -> Option<FontFamily> {
        let mut fonts = FnvHashMap::default();
        fonts.insert(TextStyle::default(), Font::new(core, path, idx)?);
        Some(FontFamily { family, fonts })
    }
}

struct Font {
    raster: RasterFace,
    shaper: HbFont,
}

impl Font {
    fn new(core: &mut RasterCore, path: &CStr, idx: u32) -> Option<Font> {
        let raster = core.new_face(path, idx)?;
        let shaper = HbFont::new(path, idx)?;
        Some(Font { raster, shaper })
    }
}
