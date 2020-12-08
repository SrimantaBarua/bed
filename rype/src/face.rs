// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::collections::HashMap;
use std::fmt;

use super::cmap::Cmap;
use super::error::*;
use super::gasp::Gasp;
use super::gsub::Gsub;
use super::head::Head;
use super::hhea::Hhea;
use super::hmtx::Hmtx;
use super::maxp::Maxp;
use super::rcbuffer::RcBuf;
use super::types::*;

/// A face within an OpenType file
pub struct Face {
    data: RcBuf,                 // Data for file
    face_offset: usize,          // Offset into file for this face
    tables: HashMap<Tag, RcBuf>, // Hashmap of tables keyed by tag
    head: Head,
    hhea: Hhea,
    maxp: Maxp,
    hmtx: Hmtx,
    cmap: Cmap,
    face_type: FaceType,
    gsub: Option<Gsub>,
}

impl Face {
    /// Initialize face structure
    pub(super) fn load(data: RcBuf, offset: usize) -> Result<Face> {
        let slice = data.as_ref();
        let sfnt_version = get_tag(slice, offset)?;
        let num_tables = get_u16(slice, offset + offsets::NUM_TABLES)? as usize;
        let mut record_offset = offset + offsets::TABLE_RECORDS;
        let mut tables = HashMap::new();

        for _ in 0..num_tables {
            let tag = get_tag(slice, record_offset)?;
            let table_offset = get_u32(slice, record_offset + offsets::TABLE_OFFSET)? as usize;
            let table_size = get_u32(slice, record_offset + offsets::TABLE_SIZE)? as usize;
            if table_offset + table_size > slice.len() {
                return Err(Error::Invalid);
            }
            let table_data = data.slice(table_offset..table_offset + table_size);
            tables.insert(tag, table_data);
            record_offset += sizes::TABLE_RECORD;
        }

        let head = Tag::from_str("head")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| Head::load(data.clone()))?;
        let hhea = Tag::from_str("hhea")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| Hhea::load(data.clone()))?;
        let maxp = Tag::from_str("maxp")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| Maxp::load(data.clone()))?;
        let hmtx = Tag::from_str("hmtx")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| {
                Hmtx::load(
                    data.clone(),
                    maxp.num_glyphs as usize,
                    hhea.num_h_metrics as usize,
                )
            })?;
        let cmap = Tag::from_str("cmap")
            .and_then(|t| tables.get(&t).ok_or(Error::Invalid))
            .and_then(|data| Cmap::load(data.clone()))?;

        let face_type = match sfnt_version {
            Tag(0x00010000) => {
                let gasp = Tag::from_str("gasp")
                    .ok()
                    .and_then(|t| tables.get(&t))
                    .map(|d| Gasp::load(d.clone()).expect("failed to load gasp"));
                FaceType::TTF { gasp }
            }
            Tag(0x4F54544F) => FaceType::CFF,
            _ => return Err(Error::Invalid),
        };

        let gsub = Tag::from_str("GSUB")
            .ok()
            .and_then(|t| tables.get(&t))
            .map(|d| Gsub::load(d.clone()).expect("failed to load GSUB"));

        Ok(Face {
            data,
            face_offset: offset,
            tables,
            head,
            hhea,
            maxp,
            hmtx,
            cmap,
            face_type,
            gsub,
        })
    }
}

impl fmt::Debug for Face {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Face")
            .field(
                "tables",
                &self
                    .tables
                    .keys()
                    .map(|k| format!("{:?}", k))
                    .collect::<Vec<_>>(),
            )
            .field("head", &self.head)
            .field("hhea", &self.hhea)
            .field("maxp", &self.maxp)
            .field("cmap", &self.cmap)
            //.field("hmtx", &self.hmtx)
            .field("face_offset", &self.face_offset)
            .field("face_type", &self.face_type)
            .field("GSUB", &self.gsub)
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
