// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ffi::{CString, OsString};
use std::os::windows::ffi::OsStringExt;

use com_wrapper::ComWrapper;
use directwrite::enums::{FontStretch, FontStyle, FontWeight};
use directwrite::factory::Factory;
use directwrite::font::Font;
use directwrite::font_collection::FontCollection;
use winapi::um::dwrite::IDWriteLocalFontFileLoader;
use winapi::Interface;
use fnv::FnvHashSet;

use crate::style::{TextSlant, TextWeight};

pub(super) struct FontSource {
    factory: Factory,
    font_collection: FontCollection,
}

impl FontSource {
    pub(super) fn new() -> Option<FontSource> {
        let factory = Factory::new().ok()?;
        let font_collection = FontCollection::system_font_collection(&factory, false).ok()?;
        Some(FontSource {
            factory: factory,
            font_collection: font_collection,
        })
    }

    pub(super) fn find_match(&mut self, pattern: &mut Pattern) -> Option<(String, CString, u32)> {
        if let Some(charset) = &pattern.charset {
            for family in self.font_collection.all_families() {
                if let Some(font_list) =
                    family.matching_fonts(pattern.weight, FontStretch::Normal, pattern.slant)
                {
                    for font in font_list.all_fonts() {
                        let mut found = true;
                        for c in charset.set.iter() {
                            if !font.has_character(*c) {
                                found = false;
                                break;
                            }
                        }
                        if found {
                            // TODO: Get family name
                            let family_name = if let Some(family_str) = &pattern.family {
                                family_str.to_owned()
                            } else {
                                "fallback".to_owned()
                            };
                            return return_from_font(family_name, font);
                        }
                    }
                }
            }
            None
        } else {
            if let Some(family_str) = &pattern.family {
                let idx = self.font_collection.find_family_by_name(family_str)?;
                let family = self.font_collection.family(idx)?;
                let font = family.first_matching_font(
                    pattern.weight,
                    FontStretch::Normal,
                    pattern.slant,
                )?;
                return_from_font(family_str.to_owned(), font)
            } else {
                unimplemented!()
            }
        }
    }
}

fn return_from_font(family_name: String, font: Font) -> Option<(String, CString, u32)> {
    let face = font.create_face().ok()?;
    let index = face.index();
    let files = face.files().ok()?;
    assert!(files.len() == 1); // TODO handle this correctly
    let file = &files[0];

    let path = unsafe {
        let raw_file = &mut *file.get_raw();

        let mut key_ptr = std::ptr::null();
        let mut key_len = 0;
        raw_file.GetReferenceKey(&mut key_ptr, &mut key_len);

        let mut loader_ptr = std::ptr::null_mut();
        raw_file.GetLoader(&mut loader_ptr);
        let loader = &mut *loader_ptr;

        let guid = IDWriteLocalFontFileLoader::uuidof();
        let mut local_loader_ptr = std::ptr::null_mut();
        loader.QueryInterface(&guid, &mut local_loader_ptr);
        let local_loader = &mut *(local_loader_ptr as *mut IDWriteLocalFontFileLoader);

        let mut path_len = 0;
        local_loader.GetFilePathLengthFromKey(key_ptr, key_len, &mut path_len);
        let mut path_buf = vec![0; (path_len + 1) as usize];
        local_loader.GetFilePathFromKey(key_ptr, key_len, path_buf.as_mut_ptr(), path_len * 2);

        OsString::from_wide(&path_buf)
            .into_string()
            .ok()
            .and_then(|mut s| {
                while let Some(c) = s.pop() {
                    if c != '\u{0}' {
                        s.push(c);
                        break;
                    }
                }
                let ret = CString::new(s).ok();
                ret
            })?
    };

    Some((family_name, path, index))
}

pub(super) struct Pattern {
    family: Option<String>,
    weight: FontWeight,
    slant: FontStyle,
    charset: Option<Charset>,
}

impl Pattern {
    pub(super) fn new() -> Option<Pattern> {
        Some(Pattern {
            family: None,
            weight: FontWeight::NORMAL,
            slant: FontStyle::Normal,
            charset: None,
        })
    }

    pub(super) fn set_family(&mut self, name: &str) -> bool {
        self.family = Some(name.to_owned());
        true
    }

    pub(super) fn set_weight(&mut self, weight: TextWeight) -> bool {
        self.weight = weight_to_dw(weight);
        true
    }

    pub(super) fn set_slant(&mut self, slant: TextSlant) -> bool {
        self.slant = slant_to_dw(slant);
        true
    }

    pub(super) fn add_charset(&mut self, charset: Charset) -> bool {
        self.charset = Some(charset);
        true
    }
}

pub(super) struct Charset {
    set: FnvHashSet<char>,
}

impl Charset {
    pub(super) fn new() -> Option<Charset> {
        Some(Charset {
            set: FnvHashSet::default(),
        })
    }

    pub(super) fn add_char(&mut self, c: char) -> bool {
        self.set.insert(c)
    }
}

/// Get directwrite weight for our weight type
fn weight_to_dw(weight: TextWeight) -> FontWeight {
    match weight {
        TextWeight::Light => FontWeight::LIGHT,
        TextWeight::Medium => FontWeight::NORMAL,
        TextWeight::Bold => FontWeight::BOLD,
    }
}

/// Get directwrite slant for our slant type
fn slant_to_dw(slant: TextSlant) -> FontStyle {
    match slant {
        TextSlant::Italic => FontStyle::Italic,
        TextSlant::Oblique => FontStyle::Oblique,
        TextSlant::Roman => FontStyle::Normal,
    }
}
