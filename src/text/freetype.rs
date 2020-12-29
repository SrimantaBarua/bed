// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::ops::Drop;
use std::{ptr, slice};

use euclid::{size2, Point2D, Size2D};
use freetype::freetype::{
    FT_Done_Face, FT_Done_FreeType, FT_Face, FT_Get_Char_Index, FT_Init_FreeType, FT_Library,
    FT_Load_Glyph, FT_New_Face, FT_Set_Pixel_Sizes, FT_Set_Transform, FT_Vector, FT_LOAD_DEFAULT,
    FT_LOAD_RENDER,
};

use crate::common::PixelSize;
use crate::style::TextSize;

use super::f26_6;

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

    pub(super) fn new_face(&self, path: &CStr, idx: u32) -> Option<RasterFont> {
        let mut face = MaybeUninit::uninit();
        let ret = unsafe { FT_New_Face(self.ft_lib, path.as_ptr(), idx as _, face.as_mut_ptr()) };
        if ret != 0 {
            None
        } else {
            Some(RasterFont {
                face: unsafe { face.assume_init() },
            })
        }
    }
}

pub(super) struct RasterFont {
    face: FT_Face,
}

// FreeType is thread-safe
unsafe impl Send for RasterFont {}

impl std::ops::Drop for RasterFont {
    fn drop(&mut self) {
        unsafe { FT_Done_Face(self.face) };
    }
}

impl RasterFont {
    pub(super) fn raster(
        &mut self,
        origin: Point2D<f26_6, PixelSize>,
        gid: u32,
        size: TextSize,
    ) -> Option<RasterizedGlyph> {
        self.set_pixel_size(size);
        unsafe {
            let mut vector = FT_Vector {
                x: origin.x.to_raw() as _,
                y: origin.y.to_raw() as _,
            };
            FT_Set_Transform(self.face, ptr::null_mut(), &mut vector as *mut _);
            if FT_Load_Glyph(self.face, gid, FT_LOAD_RENDER as i32) != 0 {
                return None;
            }
        }
        unsafe {
            let metrics = (&*(&*self.face).size).metrics;
            let (asc, desc) = (metrics.ascender, metrics.descender);
            let slot = &*(&*self.face).glyph;
            let bitmap = slot.bitmap;
            let bitmap_left = slot.bitmap_left;
            let bitmap_top = slot.bitmap_top;
            let rows = bitmap.rows;
            let width = bitmap.width;
            let ptr = bitmap.buffer;
            let buffer = slice::from_raw_parts(ptr, rows as usize * width as usize);
            let advance = slot.advance;
            Some(RasterizedGlyph {
                buffer,
                metrics: GlyphMetrics {
                    ascender: f26_6::from_raw(asc as i32),
                    descender: f26_6::from_raw(desc as i32),
                    size: size2(width, rows),
                    bearing: size2(bitmap_left, bitmap_top),
                    advance: size2(
                        f26_6::from_raw(advance.x as _),
                        f26_6::from_raw(advance.y as _),
                    ),
                },
            })
        }
    }

    pub(super) fn get_glyph_metrics(&mut self, gid: u32, size: TextSize) -> Option<GlyphMetrics> {
        self.set_pixel_size(size);
        unsafe {
            let mut vector = FT_Vector { x: 0, y: 0 };
            FT_Set_Transform(self.face, ptr::null_mut(), &mut vector as *mut _);
            if FT_Load_Glyph(self.face, gid, FT_LOAD_DEFAULT as i32) != 0 {
                return None;
            }
        }
        unsafe {
            let metrics = (&*(&*self.face).size).metrics;
            let (asc, desc) = (metrics.ascender, metrics.descender);
            let slot = &*(&*self.face).glyph;
            let bitmap = slot.bitmap;
            let bitmap_left = slot.bitmap_left;
            let bitmap_top = slot.bitmap_top;
            let rows = bitmap.rows;
            let width = bitmap.width;
            let advance = slot.advance;
            Some(GlyphMetrics {
                ascender: f26_6::from_raw(asc as i32),
                descender: f26_6::from_raw(desc as i32),
                size: size2(width, rows),
                bearing: size2(bitmap_left, bitmap_top),
                advance: size2(
                    f26_6::from_raw(advance.x as _),
                    f26_6::from_raw(advance.y as _),
                ),
            })
        }
    }

    pub(super) fn get_metrics(&mut self, size: TextSize) -> FontMetrics {
        self.set_pixel_size(size);
        let (face, metrics) = unsafe {
            let face = &*self.face;
            let metrics = (&*face.size).metrics;
            (face, metrics)
        };
        let scale = self.units_to_pixels_scale(size);
        let (under_pos, under_thick) = (face.underline_position, face.underline_thickness);
        let (asc, desc) = (metrics.ascender, metrics.descender);
        FontMetrics {
            ascender: f26_6::from_raw(asc as i32),
            descender: f26_6::from_raw(desc as i32),
            underline_pos: f26_6::from(under_pos as f32 * scale),
            underline_thickness: f26_6::from(under_thick as f32 * scale),
        }
    }

    pub(super) fn has_glyph_for_char(&self, c: char) -> bool {
        unsafe { FT_Get_Char_Index(self.face, c as _) != 0 }
    }

    pub(super) fn get_glyph_for_char(&self, c: char) -> u32 {
        unsafe { FT_Get_Char_Index(self.face, c as _) }
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
    pub(super) buffer: &'a [u8],
    pub(super) metrics: GlyphMetrics,
}

pub(crate) struct FontMetrics {
    pub(crate) ascender: f26_6,
    pub(crate) descender: f26_6,
    pub(crate) underline_pos: f26_6,
    pub(crate) underline_thickness: f26_6,
}

#[derive(Clone, Debug)]
pub(crate) struct GlyphMetrics {
    pub(crate) ascender: f26_6,
    pub(crate) descender: f26_6,
    pub(crate) size: Size2D<u32, PixelSize>,
    pub(crate) bearing: Size2D<i32, PixelSize>,
    pub(crate) advance: Size2D<f26_6, PixelSize>,
}
