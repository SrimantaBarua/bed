// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::{CStr, CString};
use std::ops::Drop;
use std::ptr;

use fontconfig::fontconfig::{
    FcCharSet, FcCharSetAddChar, FcCharSetCreate, FcCharSetDestroy, FcConfig, FcConfigSubstitute,
    FcDefaultSubstitute, FcFontMatch, FcInitLoadConfigAndFonts, FcMatchPattern, FcPattern,
    FcPatternAddCharSet, FcPatternAddInteger, FcPatternAddString, FcPatternCreate,
    FcPatternDestroy, FcPatternGetInteger, FcPatternGetString, FcResultMatch, FC_SLANT_ITALIC,
    FC_SLANT_OBLIQUE, FC_SLANT_ROMAN, FC_WEIGHT_BOLD, FC_WEIGHT_LIGHT, FC_WEIGHT_MEDIUM,
};

use crate::style::{TextSlant, TextWeight};

pub(super) struct FontSource {
    raw: *mut FcConfig,
}

// FreeType is thread-safe
unsafe impl Send for FontSource {}

impl FontSource {
    pub(super) fn new() -> Option<FontSource> {
        let ptr = unsafe { FcInitLoadConfigAndFonts() };
        if ptr.is_null() {
            None
        } else {
            Some(FontSource { raw: ptr })
        }
    }

    pub(super) fn find_match(&mut self, pattern: &mut Pattern) -> Option<(String, CString, u32)> {
        let file = b"file\0";
        let family = b"family\0";
        let index = b"index\0";
        unsafe {
            FcConfigSubstitute(self.raw, pattern.raw, FcMatchPattern);
            FcDefaultSubstitute(pattern.raw);
            let font = self.font_match(&pattern)?;
            let family = font
                .get_string(family)
                .and_then(|f| f.to_str().map(|s| s.to_owned()).ok())?;
            let file = font.get_string(file)?;
            let idx = font.get_integer(index)?;
            Some((family, file, idx as u32))
        }
    }

    fn font_match(&mut self, pattern: &Pattern) -> Option<Pattern> {
        let mut res = 0u32;
        unsafe {
            let font = FcFontMatch(self.raw, pattern.raw, &mut res as *mut _);
            if font.is_null() {
                None
            } else {
                Some(Pattern { raw: font })
            }
        }
    }
}

pub(super) struct Pattern {
    raw: *mut FcPattern,
}

impl Pattern {
    pub(super) fn new() -> Option<Pattern> {
        let ptr = unsafe { FcPatternCreate() };
        if ptr.is_null() {
            None
        } else {
            Some(Pattern { raw: ptr })
        }
    }

    pub(super) fn set_family(&mut self, name: &str) -> bool {
        let c_name = CString::new(name).unwrap();
        let s = b"family\0".as_ptr() as *const _;
        unsafe { FcPatternAddString(self.raw, s, c_name.as_ptr() as *const _) != 0 }
    }

    pub(super) fn set_weight(&mut self, weight: TextWeight) -> bool {
        let s = b"weight\0".as_ptr() as *const _;
        unsafe { FcPatternAddInteger(self.raw, s, weight_to_fc(weight)) != 0 }
    }

    pub(super) fn set_slant(&mut self, slant: TextSlant) -> bool {
        let s = b"slant\0".as_ptr() as *const _;
        unsafe { FcPatternAddInteger(self.raw, s, slant_to_fc(slant)) != 0 }
    }

    pub(super) fn add_charset(&mut self, charset: Charset) -> bool {
        let s = b"charset\0";
        unsafe { FcPatternAddCharSet(self.raw, s.as_ptr() as *const i8, charset.raw) != 0 }
    }

    fn get_string(&self, obj: &[u8]) -> Option<CString> {
        let objc = obj.as_ptr() as *const _;
        let mut ptr = ptr::null_mut();
        unsafe {
            if FcPatternGetString(self.raw, objc, 0, &mut ptr) != FcResultMatch {
                None
            } else {
                Some(CStr::from_ptr(ptr as *const _).to_owned())
            }
        }
    }

    fn get_integer(&self, obj: &[u8]) -> Option<i32> {
        let objc = obj.as_ptr() as *const _;
        let mut ival = 0i32;
        unsafe {
            if FcPatternGetInteger(self.raw, objc, 0, &mut ival) != FcResultMatch {
                None
            } else {
                Some(ival)
            }
        }
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe {
            FcPatternDestroy(self.raw);
        }
    }
}

pub(super) struct Charset {
    raw: *mut FcCharSet,
}

impl Charset {
    pub(super) fn new() -> Option<Charset> {
        let ptr = unsafe { FcCharSetCreate() };
        if ptr.is_null() {
            None
        } else {
            Some(Charset { raw: ptr })
        }
    }

    pub(super) fn add_char(&mut self, c: char) -> bool {
        unsafe { FcCharSetAddChar(self.raw, c as u32) != 0 }
    }
}

impl Drop for Charset {
    fn drop(&mut self) {
        unsafe {
            FcCharSetDestroy(self.raw);
        }
    }
}

// Get Fontconfig weight for our weight type
fn weight_to_fc(weight: TextWeight) -> i32 {
    match weight {
        TextWeight::Light => FC_WEIGHT_LIGHT,
        TextWeight::Medium => FC_WEIGHT_MEDIUM,
        TextWeight::Bold => FC_WEIGHT_BOLD,
    }
}

// Get Fontconfig slant for our slant type
fn slant_to_fc(slant: TextSlant) -> i32 {
    match slant {
        TextSlant::Italic => FC_SLANT_ITALIC,
        TextSlant::Oblique => FC_SLANT_OBLIQUE,
        TextSlant::Roman => FC_SLANT_ROMAN,
    }
}
