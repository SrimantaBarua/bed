// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::collections::HashMap;
use std::fmt;

use super::cmap::Cmap;
use super::error::*;
use super::gasp::Gasp;
use super::gdef::Gdef;
use super::gpos::Gpos;
use super::gsub::Gsub;
use super::head::Head;
use super::hhea::Hhea;
use super::hmtx::Hmtx;
use super::kern::Kern;
use super::maxp::Maxp;
use super::types::*;

/// A face within an OpenType file
pub struct Face {
    tables: Vec<Tag>, // Hashmap of tables keyed by tag
    head: Head,
    hhea: Hhea,
    maxp: Maxp,
    hmtx: Hmtx,
    cmap: Cmap,
    face_type: FaceType,
    gsub: Option<Gsub>,
    gpos: Option<Gpos>,
    kern: Option<Kern>,
    gdef: Option<Gdef>,
}

impl Face {
    /// Load face at given index from font file
    pub fn open<P: AsRef<std::path::Path>>(path: P, index: usize) -> Result<Face> {
        let data = std::fs::read(path)?;
        // Is this a font collection or a single face?
        let tag = get_tag(&data, 0)?;
        if tag == Tag::from_str("ttcf")? {
            let num_fonts = get_u32(&data, 8)? as usize;
            if num_fonts <= index {
                Err(Error::Invalid)
            } else {
                let offset = get_u32(&data, 12 + index * 4)? as usize;
                Self::load_face(&data, offset)
            }
        } else if index != 0 {
            Err(Error::Invalid)
        } else if tag != Tag(0x00010000) && tag != Tag::from_str("OTTO")? {
            Err(Error::Invalid)
        } else {
            Self::load_face(&data, 0)
        }
    }

    /// Initialize face structure
    fn load_face(data: &[u8], offset: usize) -> Result<Face> {
        let sfnt_version = get_tag(data, offset)?;
        let num_tables = get_u16(data, offset + offsets::NUM_TABLES)? as usize;
        let mut record_offset = offset + offsets::TABLE_RECORDS;
        let mut tables = HashMap::new();

        for _ in 0..num_tables {
            let tag = get_tag(data, record_offset)?;
            let table_offset = get_u32(data, record_offset + offsets::TABLE_OFFSET)? as usize;
            let table_size = get_u32(data, record_offset + offsets::TABLE_SIZE)? as usize;
            if table_offset + table_size > data.len() {
                return Err(Error::Invalid);
            }
            let table_data = &data[table_offset..table_offset + table_size];
            tables.insert(tag, table_data);
            record_offset += sizes::TABLE_RECORD;
        }

        let head = Tag::from_str("head")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| Head::load(data))?;
        let hhea = Tag::from_str("hhea")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| Hhea::load(data))?;
        let maxp = Tag::from_str("maxp")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| Maxp::load(data))?;
        let hmtx = Tag::from_str("hmtx")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| {
                Hmtx::load(data, maxp.num_glyphs as usize, hhea.num_h_metrics as usize)
            })?;
        let cmap = Tag::from_str("cmap")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| Cmap::load(data))?;

        let face_type = match sfnt_version {
            Tag(0x00010000) => {
                let gasp = Tag::from_str("gasp")
                    .ok()
                    .and_then(|t| tables.get(&t))
                    .map(|d| Gasp::load(d).expect("failed to load gasp"));
                FaceType::TTF { gasp }
            }
            Tag(0x4F54544F) => FaceType::CFF,
            _ => return Err(Error::Invalid),
        };

        let gsub = Tag::from_str("GSUB")
            .ok()
            .and_then(|t| tables.get(&t))
            .map(|d| Gsub::load(d).expect("failed to load GSUB"));
        let gpos = Tag::from_str("GPOS")
            .ok()
            .and_then(|t| tables.get(&t))
            .map(|d| Gpos::load(d).expect("failed to load GPOS"));
        let kern = Tag::from_str("kern")
            .ok()
            .and_then(|t| tables.get(&t))
            .map(|d| Kern::load(d).expect("failed to load kern"));
        let gdef = Tag::from_str("GDEF")
            .ok()
            .and_then(|t| tables.get(&t))
            .map(|d| Gdef::load(d).expect("failed to load GDEF"));

        Ok(Face {
            tables: tables.keys().map(|t| *t).collect::<Vec<_>>(),
            head,
            hhea,
            maxp,
            hmtx,
            cmap,
            face_type,
            gsub,
            gpos,
            kern,
            gdef,
        })
    }
}

impl fmt::Debug for Face {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Face")
            .field("tables", &self.tables)
            .field("head", &self.head)
            .field("hhea", &self.hhea)
            .field("maxp", &self.maxp)
            //.field("cmap", &self.cmap)
            //.field("hmtx", &self.hmtx)
            .field("face_type", &self.face_type)
            //.field("GSUB", &self.gsub)
            //.field("GPOS", &self.gpos)
            //.field("kern", &self.kern)
            .field("GDEF", &self.gdef)
            .finish()
    }
}

#[derive(Debug)]
enum FaceType {
    TTF { gasp: Option<Gasp> },
    CFF,
}

mod offsets {
    pub(super) const NUM_TABLES: usize = 4;
    pub(super) const TABLE_RECORDS: usize = 12;

    pub(super) const TABLE_OFFSET: usize = 8;
    pub(super) const TABLE_SIZE: usize = 12;
}

mod sizes {
    pub(super) const TABLE_RECORD: usize = 16;
}
