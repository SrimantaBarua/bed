// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::ops::Drop;
use std::slice;

use euclid::{size2, Size2D};
use freetype::freetype::{
    FT_Done_Face, FT_Done_FreeType, FT_Face, FT_Get_Char_Index, FT_Init_FreeType, FT_Library,
    FT_Load_Glyph, FT_New_Face, FT_Set_Char_Size, FT_LOAD_RENDER,
};

use super::{RasterizedGlyph, ScaledFaceMetrics};
use crate::common::DPI;
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
    pub(super) fn new() -> Option<RasterCore> {
        let mut ft = MaybeUninit::uninit();
        let ret = unsafe { FT_Init_FreeType(ft.as_mut_ptr()) };
        if ret != 0 {
            None
        } else {
            Some(RasterCore {
                ft_lib: unsafe { ft.assume_init() },
            })
        }
    }

    #[cfg(target_os = "windows")]
    pub(super) fn new_face(&self, path: &CStr, idx: u32) -> Option<RasterFace> {
        let mut face = MaybeUninit::uninit();
        let ret = unsafe { FT_New_Face(self.ft_lib, path.as_ptr(), idx as i32, face.as_mut_ptr()) };
        if ret != 0 {
            None
        } else {
            Some(RasterFace {
                face: unsafe { face.assume_init() },
            })
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub(super) fn new_face(&self, path: &CStr, idx: u32) -> Option<RasterFace> {
        let mut face = MaybeUninit::uninit();
        let ret = unsafe { FT_New_Face(self.ft_lib, path.as_ptr(), idx as i64, face.as_mut_ptr()) };
        if ret != 0 {
            None
        } else {
            Some(RasterFace {
                face: unsafe { face.assume_init() },
            })
        }
    }
}

pub(crate) struct RasterFace {
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
    pub(crate) fn raster(
        &mut self,
        gid: u32,
        size: TextSize,
        dpi: Size2D<u32, DPI>,
    ) -> Option<RasterizedGlyph> {
        self.set_char_size(size, dpi);
        let ret = unsafe { FT_Load_Glyph(self.face, gid, FT_LOAD_RENDER as i32) };
        if ret != 0 {
            return None;
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

    pub(crate) fn get_metrics(
        &mut self,
        size: TextSize,
        dpi: Size2D<u32, DPI>,
    ) -> ScaledFaceMetrics {
        self.set_char_size(size, dpi);
        let (face, metrics) = unsafe {
            let face = &*self.face;
            let metrics = (&*face.size).metrics;
            (face, metrics)
        };
        let (_, scale_height) = self.units_to_pixels_scale(size, dpi);
        let (under_pos, under_thick) = (face.underline_position, face.underline_thickness);
        let (asc, desc, adv) = (metrics.ascender, metrics.descender, metrics.max_advance);
        let under_pos = under_pos + (under_thick) / 2;
        ScaledFaceMetrics {
            ascender: ((asc as f32) / 64.0).round() as i32,
            descender: ((desc as f32) / 64.0).round() as i32,
            advance_width: ((adv as f32) / 64.0).round() as i32,
            underline_pos: (under_pos as f32 * scale_height).round() as i32,
            underline_thickness: (under_thick as f32 * scale_height).ceil() as i32,
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn has_glyph_for_char(&self, c: char) -> bool {
        unsafe { FT_Get_Char_Index(self.face, c as u32) != 0 }
    }

    #[cfg(not(target_os = "windows"))]
    pub(crate) fn has_glyph_for_char(&self, c: char) -> bool {
        unsafe { FT_Get_Char_Index(self.face, c as u64) != 0 }
    }

    #[cfg(target_os = "windows")]
    fn set_char_size(&mut self, size: TextSize, dpi: Size2D<u32, DPI>) -> bool {
        let ft_size = size.to_64th_point();
        unsafe {
            FT_Set_Char_Size(
                self.face,
                ft_size as i32,
                ft_size as i32,
                dpi.width,
                dpi.height,
            ) == 0
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn set_char_size(&mut self, size: TextSize, dpi: Size2D<u32, DPI>) -> bool {
        let ft_size = size.to_64th_point();
        unsafe { FT_Set_Char_Size(self.face, ft_size, ft_size, dpi.width, dpi.height) == 0 }
    }

    fn units_to_pixels_scale(&self, size: TextSize, dpi: Size2D<u32, DPI>) -> (f32, f32) {
        let face = unsafe { &*self.face };
        let units_per_em = face.units_per_EM as f32;
        let pix_per_em = size.to_pixel_size(dpi);
        (
            pix_per_em.width / units_per_em,
            pix_per_em.height / units_per_em,
        )
    }
}
