// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CString;

use euclid::Size2D;
use fnv::FnvHashMap;

use crate::common::PixelSize;
use crate::style::TextStyle;

mod freetype;
pub(crate) mod harfbuzz;

#[cfg(target_os = "windows")]
mod direct_write;
#[cfg(target_os = "windows")]
use self::direct_write as source;

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
mod fontconfig;
#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
use self::fontconfig as source;

use self::freetype::RasterCore;
pub(crate) use self::freetype::RasterFace;
use self::harfbuzz::{HbBuffer, HbFont};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct FaceKey(u16);

pub(crate) struct Face {
    pub(crate) raster: RasterFace,
    pub(crate) shaper: HbFont,
}

impl Face {
    fn new(core: &RasterCore, path: CString, idx: u32) -> Option<Face> {
        let raster = core.new_face(&path, idx)?;
        let shaper = HbFont::new(&path, idx)?;
        Some(Face {
            raster: raster,
            shaper: shaper,
        })
    }
}

struct FaceFamily {
    name: String,
    faces: FnvHashMap<TextStyle, Face>,
}

impl FaceFamily {
    fn empty(name: String) -> FaceFamily {
        FaceFamily {
            name: name,
            faces: FnvHashMap::default(),
        }
    }

    fn set_face(&mut self, style: TextStyle, face: Face) {
        self.faces.insert(style, face);
    }

    fn get_face_mut(&mut self, style: &TextStyle) -> Option<&mut Face> {
        self.faces.get_mut(&style)
    }

    fn get_face(&self, style: &TextStyle) -> Option<&Face> {
        self.faces.get(&style)
    }
}

pub(crate) struct FaceGroup {
    family: FaceFamily,
    fallbacks: Vec<FaceKey>,
}

impl FaceGroup {
    fn new(family: String, style: TextStyle, face: Face) -> FaceGroup {
        let mut family = FaceFamily::empty(family);
        family.set_face(style, face);
        FaceGroup {
            family: family,
            fallbacks: Vec::new(),
        }
    }
}

pub(crate) struct FontCore {
    path_face_map: FnvHashMap<(CString, u32), FaceKey>,
    key_face_map: FnvHashMap<FaceKey, FaceGroup>,
    next_key: u16,
    raster_core: RasterCore,
    hb_buffer: HbBuffer,
    source: source::FontSource,
}

impl FontCore {
    pub(crate) fn new() -> Option<FontCore> {
        let source = source::FontSource::new()?;
        let raster_core = RasterCore::new()?;
        let hb_buffer = HbBuffer::new()?;
        Some(FontCore {
            source: source,
            path_face_map: FnvHashMap::default(),
            key_face_map: FnvHashMap::default(),
            raster_core: raster_core,
            hb_buffer: hb_buffer,
            next_key: 0,
        })
    }

    pub(crate) fn find(&mut self, family: &str) -> Option<FaceKey> {
        let default_style = TextStyle::default();
        for (key, group) in self.key_face_map.iter() {
            if group.family.name == family {
                return Some(*key);
            }
        }

        let mut pattern = source::Pattern::new()?;
        if !pattern.set_family(family)
            || !pattern.set_slant(default_style.slant)
            || !pattern.set_weight(default_style.weight)
        {
            return None;
        }
        let (family, path, idx) = self.source.find_match(&mut pattern)?;

        if let Some(key) = self.path_face_map.get(&(path.clone(), idx)) {
            Some(*key)
        } else {
            for (key, group) in self.key_face_map.iter() {
                if group.family.name == family {
                    return Some(*key);
                }
            }

            let key = FaceKey(self.next_key);
            let face = Face::new(&self.raster_core, path.clone(), idx)?;
            self.key_face_map
                .insert(key, FaceGroup::new(family, default_style, face));
            self.path_face_map.insert((path, idx), key);
            self.next_key += 1;
            Some(key)
        }
    }

    pub(crate) fn find_for_char(&mut self, base: FaceKey, c: char) -> Option<FaceKey> {
        let default_style = TextStyle::default();

        let group = self.key_face_map.get(&base)?;
        let face = group.family.get_face(&default_style)?;
        if face.raster.has_glyph_for_char(c) {
            return Some(base);
        }

        for key in &group.fallbacks {
            let group = self.key_face_map.get(&key)?;
            let face = group.family.get_face(&default_style)?;
            if face.raster.has_glyph_for_char(c) {
                return Some(*key);
            }
        }

        let mut pattern = source::Pattern::new()?;
        let mut charset = source::Charset::new()?;
        charset.add_char(c);
        if !pattern.set_family(&group.family.name)
            || !pattern.set_slant(default_style.slant)
            || !pattern.set_weight(default_style.weight)
            || !pattern.add_charset(charset)
        {
            return None;
        }
        let (family, path, idx) = self.source.find_match(&mut pattern)?;

        let key = FaceKey(self.next_key);
        let face = Face::new(&self.raster_core, path, idx)?;
        if !face.raster.has_glyph_for_char(c) {
            return None;
        }

        let group = self.key_face_map.get_mut(&base)?;
        group.fallbacks.push(key);
        self.key_face_map
            .insert(key, FaceGroup::new(family, default_style, face));
        self.next_key += 1;
        Some(key)
    }

    pub(crate) fn get(
        &mut self,
        key: FaceKey,
        style: TextStyle,
    ) -> Option<(&mut HbBuffer, &mut Face)> {
        let hb_buffer = &mut self.hb_buffer;
        let group = self.key_face_map.get_mut(&key)?;
        if group.family.get_face(&style).is_some() {
            return Some((hb_buffer, group.family.get_face_mut(&style)?));
        }
        let mut pattern = source::Pattern::new()?;
        if !pattern.set_family(&group.family.name)
            || !pattern.set_slant(style.slant)
            || !pattern.set_weight(style.weight)
        {
            return None;
        }
        let (_, path, idx) = self.source.find_match(&mut pattern)?;
        let face = Face::new(&self.raster_core, path, idx)?;
        group.family.set_face(style, face);
        Some((hb_buffer, group.family.get_face_mut(&style)?))
    }
}

#[derive(Clone)]
pub(crate) struct RasterizedGlyph<'a> {
    pub(crate) size: Size2D<u32, PixelSize>,
    pub(crate) bearing: Size2D<i32, PixelSize>,
    pub(crate) buffer: &'a [u8],
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub(crate) struct ScaledFaceMetrics {
    pub(crate) ascender: i32,
    pub(crate) descender: i32,
    pub(crate) advance_width: i32,
    pub(crate) underline_pos: i32,
    pub(crate) underline_thickness: i32,
}
