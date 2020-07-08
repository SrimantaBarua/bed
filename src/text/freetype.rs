// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::ops::Drop;
use std::slice;

use euclid::{size2, Size2D};
use freetype::freetype::{
    FT_Done_Face, FT_Done_FreeType, FT_Face, FT_Get_Char_Index, FT_Init_FreeType, FT_Library,
    FT_Load_Glyph, FT_New_Face, FT_Set_Pixel_Sizes, FT_LOAD_RENDER,
};

use crate::common::PixelSize;
use crate::style::TextSize;

pub(super) struct RasterCore {
    ft_lib: FT_Library,
}

// FreeType is thread-safe
unsafe impl Send for RasterCore {}

impl Drop for RasterCore {
    fn drop(&mut self) {
        unsafe { FT_Done_FreeType(self.ft_lib) };
    }
}

impl RasterCore {
    pub(super) fn new() -> RasterCore {
        let mut ft = MaybeUninit::uninit();
        let ret = unsafe { FT_Init_FreeType(ft.as_mut_ptr()) };
        assert!(ret == 0, "Failed to initialize FreeType");
        RasterCore {
            ft_lib: unsafe { ft.assume_init() },
        }
    }

    pub(super) fn new_face(&self, path: &CStr, idx: u32) -> Option<RasterFace> {
        let mut face = MaybeUninit::uninit();
        let ret = unsafe { FT_New_Face(self.ft_lib, path.as_ptr(), idx as _, face.as_mut_ptr()) };
        if ret != 0 {
            None
        } else {
            Some(RasterFace {
                face: unsafe { face.assume_init() },
            })
        }
    }
}

pub(super) struct RasterFace {
    face: FT_Face,
}

// FreeType is thread-safe
unsafe impl Send for RasterFace {}

impl std::ops::Drop for RasterFace {
    fn drop(&mut self) {
        unsafe { FT_Done_Face(self.face) };
    }
}

impl RasterFace {
    pub(super) fn raster(&mut self, gid: u32, size: TextSize) -> Option<RasterizedGlyph> {
        self.set_pixel_size(size);
        unsafe {
            if FT_Load_Glyph(self.face, gid, FT_LOAD_RENDER as i32) != 0 {
                return None;
            }
        }
        unsafe {
            let slot = &*(&*self.face).glyph;
            let bitmap = slot.bitmap;
            let bitmap_left = slot.bitmap_left;
            let bitmap_top = slot.bitmap_top;
            let rows = bitmap.rows;
            let width = bitmap.width;
            let ptr = bitmap.buffer;
            let buffer = slice::from_raw_parts(ptr, rows as usize * width as usize);
            Some(RasterizedGlyph {
                size: size2(width, rows),
                bearing: size2(bitmap_left, bitmap_top),
                buffer,
            })
        }
    }

    pub(super) fn get_metrics(&mut self, size: TextSize) -> ScaledFaceMetrics {
        self.set_pixel_size(size);
        let (face, metrics) = unsafe {
            let face = &*self.face;
            let metrics = (&*face.size).metrics;
            (face, metrics)
        };
        let scale = self.units_to_pixels_scale(size);
        let (under_pos, under_thick) = (face.underline_position, face.underline_thickness);
        let (asc, desc, adv) = (metrics.ascender, metrics.descender, metrics.max_advance);
        let under_pos = under_pos + (under_thick) / 2;
        ScaledFaceMetrics {
            ascender: ((asc as f32) / 64.0).round() as i32,
            descender: ((desc as f32) / 64.0).round() as i32,
            advance_width: ((adv as f32) / 64.0).round() as i32,
            underline_pos: (under_pos as f32 * scale).round() as i32,
            underline_thickness: (under_thick as f32 * scale).ceil() as i32,
        }
    }

    pub(super) fn has_glyph_for_char(&self, c: char) -> bool {
        unsafe { FT_Get_Char_Index(self.face, c as _) != 0 }
    }

    fn set_pixel_size(&mut self, size: TextSize) -> bool {
        let pix_size = size.to_i64();
        unsafe { FT_Set_Pixel_Sizes(self.face, pix_size as _, pix_size as _) == 0 }
    }

    fn units_to_pixels_scale(&self, size: TextSize) -> f32 {
        let face = unsafe { &*self.face };
        let units_per_em = face.units_per_EM as f32;
        let pix_per_em = size.to_f32();
        pix_per_em / units_per_em
    }
}

pub(super) struct RasterizedGlyph<'a> {
    pub(super) size: Size2D<u32, PixelSize>,
    pub(super) bearing: Size2D<i32, PixelSize>,
    pub(super) buffer: &'a [u8],
}

pub(super) struct ScaledFaceMetrics {
    pub(super) ascender: i32,
    pub(super) descender: i32,
    pub(super) advance_width: i32,
    pub(super) underline_pos: i32,
    pub(super) underline_thickness: i32,
}
