// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;
use std::rc::Rc;

use fnv::FnvHashMap;
use geom::{size2, vec2, Size2D};

use super::cmap::Cmap;
use super::common::{GlyphInfo, ScaledGlyphInfo};
use super::direction::Direction;
use super::error::*;
use super::features::*;
use super::gasp::Gasp;
use super::gdef::Gdef;
use super::glyf::Glyf;
use super::gpos::Gpos;
use super::gsub::Gsub;
use super::head::Head;
use super::hhea::Hhea;
use super::hmtx::Hmtx;
use super::kern::Kern;
use super::loca::Loca;
use super::maxp::Maxp;
use super::os2::Os2;
use super::types::*;
use super::Script;

/// A face that has been scaled
#[derive(Debug)]
pub struct ScaledFace {
    scale: Size2D<f32>,
    face_inner: Rc<FaceInner>,
}

impl ScaledFace {
    pub fn shape<S: AsRef<str>>(
        &self,
        text: &S,
        script: Script,
        direction: Direction,
    ) -> Result<(Vec<char>, Vec<ScaledGlyphInfo>)> {
        let codepoints = text.as_ref().chars().collect::<Vec<_>>();
        let mut glyph_ids = codepoints
            .iter()
            .map(|cp| self.face_inner.cmap.glyph_id_for_codepoint(*cp as u32))
            .collect::<Vec<_>>();
        let features = DEFAULT_FEATURES
            .iter()
            .map(|f| f.tag())
            .chain(direction.features().iter().map(|f| f.tag()))
            .collect();
        if let Some(gsub) = &self.face_inner.gsub {
            gsub.substitute(&mut glyph_ids, script, &features);
        }
        let glyph_infos = glyph_ids.iter().map(|g| {
            let hor_metrics = self.face_inner.hmtx.get_metrics(*g);
            let bbox = match &self.face_inner.face_type {
                FaceType::TTF { glyf, .. } => glyf.glyph_bbox(*g),
            };
            GlyphInfo {
                glyph: *g,
                size: size2(bbox.max.x - bbox.min.x, bbox.max.y - bbox.min.y).cast(),
                bearing: vec2(hor_metrics.lsb, bbox.max.y),
                offset: vec2(0, 0),
                advance: vec2(hor_metrics.advance_width, 0),
            }
        });
        /*
        if let Some(gpos) = &self.face_inner.gpos {
            gpos.apply(&mut glyph_infos, script, &features);
        }
        */
        // Scale all glyph data
        let scaled_glyph_infos = glyph_infos.map(|g| g.scale(self.scale)).collect();
        Ok((codepoints, scaled_glyph_infos))
    }
}

/// A face within an OpenType file
#[derive(Debug)]
pub struct Face(Rc<FaceInner>);

impl Face {
    /// Load face at given index from font file
    pub fn open<P: AsRef<std::path::Path>>(path: P, index: usize) -> Result<Face> {
        let data = std::fs::read(path)?;
        // Is this a font collection or a single face?
        let tag = get_tag(&data, 0)?;
        if tag == Tag::from(b"ttcf") {
            let num_fonts = get_u32(&data, 8)? as usize;
            if num_fonts <= index {
                Err(Error::Invalid)
            } else {
                let offset = get_u32(&data, 12 + index * 4)? as usize;
                Self::load_face(&data, offset)
            }
        } else if index != 0 {
            Err(Error::Invalid)
        } else if tag != Tag(0x00010000) && tag != Tag::from(b"OTTO") {
            Err(Error::Invalid)
        } else {
            Self::load_face(&data, 0)
        }
    }

    /// Get scaled face
    pub fn scale(&self, point_size: u16, dpi: Size2D<u16>) -> ScaledFace {
        let point_size = point_size as f32;
        let units_per_em = self.0.head.units_per_em as f32;
        let mult = point_size / (72.0 * units_per_em);
        let scale = size2(dpi.width as f32 * mult, dpi.height as f32 * mult);
        let face_inner = self.0.clone();
        ScaledFace { scale, face_inner }
    }

    /// Initialize face structure
    fn load_face(data: &[u8], offset: usize) -> Result<Face> {
        FaceInner::load(data, offset).map(|fi| Face(Rc::new(fi)))
    }
}

pub(crate) struct FaceInner {
    tables: Vec<Tag>, // Table tags for debugging purposes
    head: Head,
    hhea: Hhea,
    maxp: Maxp,
    hmtx: Hmtx,
    cmap: Cmap,
    os2: Os2,
    face_type: FaceType,
    gsub: Option<Gsub>,
    gpos: Option<Gpos>,
    kern: Option<Kern>,
    gdef: Option<Rc<Gdef>>,
}

impl FaceInner {
    fn load(data: &[u8], offset: usize) -> Result<FaceInner> {
        let sfnt_version = get_tag(data, offset)?;
        let num_tables = get_u16(data, offset + offsets::NUM_TABLES)? as usize;
        let mut record_offset = offset + offsets::TABLE_RECORDS;
        let mut tables = FnvHashMap::default();

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

        let head = tables
            .get(&Tag::from(b"head"))
            .ok_or(Error::Invalid)
            .and_then(|data| Head::load(data))?;
        let hhea = tables
            .get(&Tag::from(b"hhea"))
            .ok_or(Error::Invalid)
            .and_then(|data| Hhea::load(data))?;
        let maxp = tables
            .get(&Tag::from(b"maxp"))
            .ok_or(Error::Invalid)
            .and_then(|data| Maxp::load(data))?;
        let hmtx = tables
            .get(&Tag::from(b"hmtx"))
            .ok_or(Error::Invalid)
            .and_then(|data| {
                Hmtx::load(data, maxp.num_glyphs as usize, hhea.num_h_metrics as usize)
            })?;
        let cmap = tables
            .get(&Tag::from(b"cmap"))
            .ok_or(Error::Invalid)
            .and_then(|data| Cmap::load(data))?;
        let os2 = tables
            .get(&Tag::from(b"OS/2"))
            .ok_or(Error::Invalid)
            .and_then(|data| Os2::load(data))?;

        const OTTO: Tag = Tag::from(b"OTTO");
        let face_type = match sfnt_version {
            Tag(0x00010000) => {
                let loca = tables
                    .get(&Tag::from(b"loca"))
                    .ok_or(Error::Invalid)
                    .and_then(|data| {
                        Loca::load(data, maxp.num_glyphs as usize, head.idx_loc_fmt)
                    })?;
                let glyf = tables
                    .get(&Tag::from(b"glyf"))
                    .ok_or(Error::Invalid)
                    .and_then(|data| Glyf::load(data, &loca))?;
                let gasp = tables
                    .get(&Tag::from(b"gasp"))
                    .map(|d| Gasp::load(d).expect("failed to load gasp"));
                FaceType::TTF { gasp, glyf }
            }
            OTTO => unimplemented!(), // OTTO = CFF
            _ => return Err(Error::Invalid),
        };

        let gdef = tables
            .get(&Tag::from(b"GDEF"))
            .map(|d| Rc::new(Gdef::load(d).expect("failed to load GDEF")));
        let gsub = tables
            .get(&Tag::from(b"GSUB"))
            .map(|d| Gsub::load(d, gdef.clone()).expect("failed to load GSUB"));
        let gpos = tables
            .get(&Tag::from(b"GPOS"))
            .map(|d| Gpos::load(d, gdef.clone()).expect("failed to load GPOS"));
        let kern = tables
            .get(&Tag::from(b"kern"))
            .map(|d| Kern::load(d).expect("failed to load kern"));

        Ok(FaceInner {
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
            os2,
        })
    }
}

impl fmt::Debug for FaceInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Face")
            .field("tables", &self.tables)
            .field("head", &self.head)
            .field("hhea", &self.hhea)
            .field("maxp", &self.maxp)
            .field("OS/2", &self.os2)
            .field("cmap", &self.cmap)
            .field("hmtx", &self.hmtx)
            .field("face_type", &self.face_type)
            .field("GSUB", &self.gsub)
            .field("GPOS", &self.gpos)
            .field("kern", &self.kern)
            .field("GDEF", &self.gdef)
            .finish()
    }
}

#[derive(Debug)]
enum FaceType {
    TTF { gasp: Option<Gasp>, glyf: Glyf },
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
