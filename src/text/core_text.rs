use std::ffi::{CStr, CString};

use crate::style::{TextSlant, TextWeight};

pub(super) struct FontSource {}

impl FontSource {
    pub(super) fn new() -> FontSource {
        FontSource {}
    }

    pub(super) fn find_match(&mut self, pattern: &mut Pattern) -> Option<(String, CString, u32)> {
        Some((
            "SF Mono".to_owned(),
            unsafe {
                CStr::from_ptr(
                    b"/Users/barua/Library/Fonts/SF-Mono-Regular.otf\0" as *const _ as *const _,
                )
                .to_owned()
            },
            0,
        ))
    }
}

pub(super) struct Pattern {}

impl Pattern {
    pub(super) fn new() -> Pattern {
        Pattern {}
    }

    pub(super) fn set_family(&mut self, name: &str) {}

    pub(super) fn set_weight(&mut self, weight: TextWeight) {}

    pub(super) fn set_slant(&mut self, slant: TextSlant) {}

    pub(super) fn add_charset(&mut self, charset: Charset) {}
}

pub(super) struct Charset {}

impl Charset {
    pub(super) fn new() -> Charset {
        Charset {}
    }

    pub(super) fn add_char(&mut self, c: char) {}
}
