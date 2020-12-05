// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

mod error;
mod ttc;
mod types;

pub use error::*;

/// Handle to a font
pub struct Font {
    data: Box<[u8]>, // Data for file
    offset: usize,   // Offset into file for start of this font
}

impl Font {
    pub fn open<P: AsRef<std::path::Path>>(path: P, index: usize) -> Result<Font> {
        let data = std::fs::read(path)?.into_boxed_slice();
        // Is this a font collection or a single face?
        let tag = types::get_tag(&data, 0)?;
        if tag == types::Tag::from_str("ttcf")? {
            ttc::TTC::load_from(&data)
                .and_then(|ttc| ttc.offset(index))
                .map(|offset| Font { data, offset })
        } else if index > 0 {
            Err(Error::FaceIndexOutOfBounds)
        } else {
            Ok(Font { data, offset: 0 })
        }
    }
}
