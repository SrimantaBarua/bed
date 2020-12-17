// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;
use crate::types::{get_i16, get_tag, get_u16, get_u32, Tag};

bitflags! {
    struct Typ : u16 {
        const RESTRICTRED_LICENSE_EMBEDDING = 0x0002;
        const PREVIEW_AND_PRINT_EMBEDDING   = 0x0004;
        const EDITABLE_EMBEDDING            = 0x0008;
        const NO_SUBSETTING                 = 0x0100;
        const BITMAP_EMBEDDING_ONLY         = 0x0200;
    }
}

bitflags! {
    struct Selection: u16 {
        const ITALIC           = 0x0001;
        const UNDERSCORE       = 0x0002;
        const NEGATIVE         = 0x0004;
        const OUTLINED         = 0x0008;
        const STRIKEOUT        = 0x0010;
        const BOLD             = 0x0020;
        const REGULAR          = 0x0040;
        const USE_TYPO_METRICS = 0x0080;
        const WWS              = 0x0100;
        const OBLIQUE          = 0x0200;
    }
}

#[derive(Debug)]
struct UnicodeRange(u32, u32, u32, u32);

#[derive(Debug)]
pub(crate) struct Os2 {
    avg_char_width: i16,
    weight_class: u16,
    width_class: u16,
    typ: Typ,
    subscript_x_size: i16,
    subscript_y_size: i16,
    subscript_x_offset: i16,
    subscript_y_offset: i16,
    superscript_x_size: i16,
    superscript_y_size: i16,
    superscript_x_offset: i16,
    superscript_y_offset: i16,
    strikeout_size: i16,
    strikeout_position: i16,
    family_class: i16,
    panose: [u8; 10],
    unicode_range: UnicodeRange,
    arch_vend_id: Tag,
    selection: Selection,
    typo_ascender: i16,
    type_descender: i16,
    typo_line_gap: i16,
    win_ascent: u16,
    win_descent: u16,
    x_height: Option<i16>,
    cap_height: Option<i16>,
    default_char: Option<u16>,
    break_char: Option<u16>,
    max_context: Option<u16>,
    lower_optical_point_size: Option<u16>,
    upper_optical_point_size: Option<u16>,
}

impl Os2 {
    pub(crate) fn load(data: &[u8]) -> Result<Os2> {
        let version = get_u16(data, 0)?;
        let avg_char_width = get_i16(data, 2)?;
        let weight_class = get_u16(data, 4)?;
        let width_class = get_u16(data, 6)?;
        let typ = Typ::from_bits_truncate(get_u16(data, 8)?);
        let subscript_x_size = get_i16(data, 10)?;
        let subscript_y_size = get_i16(data, 12)?;
        let subscript_x_offset = get_i16(data, 14)?;
        let subscript_y_offset = get_i16(data, 16)?;
        let superscript_x_size = get_i16(data, 18)?;
        let superscript_y_size = get_i16(data, 20)?;
        let superscript_x_offset = get_i16(data, 22)?;
        let superscript_y_offset = get_i16(data, 24)?;
        let strikeout_size = get_i16(data, 26)?;
        let strikeout_position = get_i16(data, 28)?;
        let family_class = get_i16(data, 30)?;
        let mut panose: [u8; 10] = Default::default();
        panose.copy_from_slice(&data[32..42]);
        let unicode_range = UnicodeRange(
            get_u32(data, 42)?,
            get_u32(data, 46)?,
            get_u32(data, 50)?,
            get_u32(data, 54)?,
        );
        let arch_vend_id = get_tag(data, 58)?;
        let selection = Selection::from_bits_truncate(get_u16(data, 62)?);
        let typo_ascender = get_i16(data, 68)?;
        let type_descender = get_i16(data, 70)?;
        let typo_line_gap = get_i16(data, 72)?;
        let win_ascent = get_u16(data, 74)?;
        let win_descent = get_u16(data, 76)?;
        let (x_height, cap_height, default_char, break_char, max_context) = if version >= 2 {
            (
                Some(get_i16(data, 86)?),
                Some(get_i16(data, 88)?),
                Some(get_u16(data, 90)?),
                Some(get_u16(data, 92)?),
                Some(get_u16(data, 94)?),
            )
        } else {
            (None, None, None, None, None)
        };
        let (lower_optical_point_size, upper_optical_point_size) = if version >= 5 {
            (Some(get_u16(data, 96)?), Some(get_u16(data, 98)?))
        } else {
            (None, None)
        };
        Ok(Os2 {
            avg_char_width,
            weight_class,
            width_class,
            typ,
            subscript_x_size,
            subscript_y_size,
            subscript_x_offset,
            subscript_y_offset,
            superscript_x_size,
            superscript_y_size,
            superscript_x_offset,
            superscript_y_offset,
            strikeout_size,
            strikeout_position,
            family_class,
            panose,
            unicode_range,
            arch_vend_id,
            selection,
            typo_ascender,
            type_descender,
            typo_line_gap,
            win_ascent,
            win_descent,
            x_height,
            cap_height,
            default_char,
            break_char,
            max_context,
            lower_optical_point_size,
            upper_optical_point_size,
        })
    }
}
