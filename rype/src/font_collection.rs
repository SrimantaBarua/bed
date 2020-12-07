// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;
use crate::face::Face;
use crate::rcbuffer::RcBuf;
use crate::types::*;
/// An OpenType font file can either be a "font collection" (e.g. *.otc) file, or contain a
/// single font. To provide a uniform interface, rype opens a font file as a `FontCollection`.
/// The `FontCollection` can then be queried for individual `Face`s.
pub struct FontCollection {
    data: RcBuf,            // Data for file
    face_offsets: Vec<u32>, // Offsets into file for each face
}

impl FontCollection {
    /// Load font collection from file
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<FontCollection> {
        let data = RcBuf::new(std::fs::read(path)?);
        // Is this a font collection or a single face?
        let tag = get_tag(&data, 0)?;
        if tag == Tag::from_str("ttcf")? {
            let num_fonts = get_u32(&data, offsets::NUM_FONTS)? as usize;
            let mut face_offsets = Vec::new();
            for i in 0..num_fonts {
                face_offsets.push(get_u32(&data, offsets::FONT_OFFSETS + i * 4)?);
            }
            Ok(FontCollection { data, face_offsets })
        } else if tag != Tag(consts::TRUETYPE) && tag != Tag::from_str("OTTO")? {
            Err(Error::Invalid)
        } else {
            Ok(FontCollection {
                data,
                face_offsets: vec![0],
            })
        }
    }

    /// Get face at given index
    pub fn get_face(&self, index: usize) -> Result<Face> {
        if index >= self.face_offsets.len() {
            Err(Error::FaceIndexOutOfBounds)
        } else {
            Face::load(self.data.clone(), self.face_offsets[index] as usize)
        }
    }
}

mod offsets {
    pub(super) const NUM_FONTS: usize = 8;
    pub(super) const FONT_OFFSETS: usize = 12;
}

mod consts {
    pub(super) const TRUETYPE: u32 = 0x00010000;
}
