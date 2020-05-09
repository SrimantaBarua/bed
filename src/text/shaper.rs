// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::font::{FaceKey, FontCore, RasterFace};
use crate::style::TextStyle;

// TODO: Evaluate performance on caching shaped words

pub(crate) struct TextShaper {
    font_core: FontCore,
}

impl TextShaper {
    pub(crate) fn new(font_core: FontCore) -> TextShaper {
        TextShaper {
            font_core: font_core,
        }
    }

    pub(crate) fn get_raster(
        &mut self,
        face_key: FaceKey,
        style: TextStyle,
    ) -> Option<&mut RasterFace> {
        self.font_core
            .get(face_key, style)
            .map(|(_, f)| &mut f.raster)
    }

    pub(crate) fn shape_line_rope(&mut self) {}

    // TODO: Write function to shape str slices
}
