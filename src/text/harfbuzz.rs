// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;
use std::slice;

use euclid::{size2, Size2D};

use crate::common::PixelSize;
use crate::style::TextSize;

use harfbuzz_sys::{
    hb_blob_create_from_file, hb_blob_destroy, hb_blob_t, hb_buffer_add_utf8,
    hb_buffer_clear_contents, hb_buffer_create, hb_buffer_destroy, hb_buffer_get_glyph_infos,
    hb_buffer_get_glyph_positions, hb_buffer_guess_segment_properties, hb_buffer_t, hb_face_create,
    hb_face_destroy, hb_font_create, hb_font_destroy, hb_font_set_scale, hb_font_t,
    hb_glyph_info_t, hb_glyph_position_t, hb_shape,
};

pub(super) fn shape<'a>(font: &HbFont, buf: &'a mut HbBuffer) -> GlyphInfoIter<'a> {
    unsafe {
        hb_shape(font.raw, buf.raw, std::ptr::null(), 0);
    }
    buf.get_info_and_pos()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GlyphInfo {
    pub(super) gid: u32,
    pub(super) cluster: u32,
    pub(super) advance: Size2D<i32, PixelSize>,
    pub(super) offset: Size2D<i32, PixelSize>,
}

pub(super) struct GlyphInfoIter<'a> {
    info: &'a [hb_glyph_info_t],
    pos: &'a [hb_glyph_position_t],
    i: usize,
}

impl<'a> Iterator for GlyphInfoIter<'a> {
    type Item = GlyphInfo;

    fn next(&mut self) -> Option<GlyphInfo> {
        if self.i == self.info.len() {
            return None;
        }
        let ret = GlyphInfo {
            gid: self.info[self.i].codepoint,
            cluster: self.info[self.i].cluster,
            advance: size2(
                (self.pos[self.i].x_advance as f32 / 64.0).round() as i32,
                (self.pos[self.i].y_advance as f32 / 64.0).round() as i32,
            ),
            offset: size2(
                (self.pos[self.i].x_offset as f32 / 64.0).round() as i32,
                (self.pos[self.i].y_offset as f32 / 64.0).round() as i32,
            ),
        };
        self.i += 1;
        Some(ret)
    }
}

pub(super) struct HbBuffer {
    raw: *mut hb_buffer_t,
}

// Harbuzz is thread-safe
unsafe impl Send for HbBuffer {}

impl HbBuffer {
    pub(super) fn new() -> HbBuffer {
        let ptr = unsafe { hb_buffer_create() };
        assert!(!ptr.is_null(), "Failed to initialize HarfBuzz buffer");
        HbBuffer { raw: ptr }
    }

    pub(super) fn clear_contents(&mut self) {
        unsafe { hb_buffer_clear_contents(self.raw) }
    }

    pub(super) fn guess_segment_properties(&mut self) {
        unsafe { hb_buffer_guess_segment_properties(self.raw) }
    }

    pub(crate) fn add_utf8(&mut self, s: &str) {
        unsafe {
            hb_buffer_add_utf8(
                self.raw,
                s.as_ptr() as *const _,
                s.len() as i32,
                0,
                s.len() as i32,
            )
        }
    }

    fn get_info_and_pos(&self) -> GlyphInfoIter {
        let mut len1 = 0u32;
        let mut len2 = 0u32;
        unsafe {
            let info_ptr = hb_buffer_get_glyph_infos(self.raw, &mut len1 as *mut _);
            let pos_ptr = hb_buffer_get_glyph_positions(self.raw, &mut len2 as *mut _);
            assert!(len1 == len2, "unequal lengths for info and position arrays");
            let info = slice::from_raw_parts(info_ptr, len1 as usize);
            let pos = slice::from_raw_parts(pos_ptr, len2 as usize);
            GlyphInfoIter { info, pos, i: 0 }
        }
    }
}

impl std::ops::Drop for HbBuffer {
    fn drop(&mut self) {
        unsafe {
            hb_buffer_destroy(self.raw);
        }
    }
}

pub(super) struct HbFont {
    raw: *mut hb_font_t,
}

// Harbuzz is thread-safe
unsafe impl Send for HbFont {}

impl std::ops::Drop for HbFont {
    fn drop(&mut self) {
        unsafe { hb_font_destroy(self.raw) }
    }
}

impl HbFont {
    pub(super) fn new(path: &CStr, idx: u32) -> Option<HbFont> {
        let blob = HbBlob::from_file(path)?;
        unsafe {
            let face = hb_face_create(blob.raw, idx);
            if face.is_null() {
                return None;
            }
            let font = hb_font_create(face);
            if font.is_null() {
                return None;
            }
            hb_face_destroy(face);
            Some(HbFont { raw: font })
        }
    }

    pub(super) fn set_scale(&mut self, size: TextSize) {
        let scale = (size.to_f32() * 64.0) as i32;
        unsafe { hb_font_set_scale(self.raw, scale, scale) }
    }
}

struct HbBlob {
    raw: *mut hb_blob_t,
}

impl std::ops::Drop for HbBlob {
    fn drop(&mut self) {
        unsafe { hb_blob_destroy(self.raw) };
    }
}

impl HbBlob {
    fn from_file(path: &CStr) -> Option<HbBlob> {
        let ptr = unsafe { hb_blob_create_from_file(path.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(HbBlob { raw: ptr })
        }
    }
}
