// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::PartialEq;
use std::fmt;
use std::ops::Range;
use std::str::Chars as SChars;

use ropey::iter::Chars as RChars;
use ropey::RopeSlice;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

use crate::common::RopeGraphemes;

#[derive(Debug, Eq)]
pub(crate) enum RopeOrStr<'a> {
    Rope(RopeSlice<'a>),
    Str(&'a str),
}

impl<'a> From<&'a str> for RopeOrStr<'a> {
    fn from(s: &'a str) -> RopeOrStr<'a> {
        RopeOrStr::Str(s)
    }
}

impl<'a> From<RopeSlice<'a>> for RopeOrStr<'a> {
    fn from(r: RopeSlice<'a>) -> RopeOrStr<'a> {
        RopeOrStr::Rope(r)
    }
}

impl<'a> PartialEq for RopeOrStr<'a> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            RopeOrStr::Rope(mr) => match other {
                RopeOrStr::Rope(or) => mr == or,
                RopeOrStr::Str(os) => mr == os,
            },
            RopeOrStr::Str(ms) => match other {
                RopeOrStr::Rope(or) => ms == or,
                RopeOrStr::Str(os) => ms == os,
            },
        }
    }
}

impl<'a> fmt::Display for RopeOrStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RopeOrStr::Rope(r) => write!(f, "{}", r),
            RopeOrStr::Str(s) => write!(f, "{}", s),
        }
    }
}

impl<'a> RopeOrStr<'a> {
    pub(crate) fn len_chars(&self) -> usize {
        match self {
            RopeOrStr::Rope(r) => r.len_chars(),
            RopeOrStr::Str(s) => s.chars().count(),
        }
    }

    pub(crate) fn chars(&self) -> RopeOrStrChars {
        match self {
            RopeOrStr::Rope(r) => RopeOrStrChars::Rope(r.chars()),
            RopeOrStr::Str(s) => RopeOrStrChars::Str(s.chars()),
        }
    }

    pub(crate) fn slice(&self, crange: Range<usize>) -> RopeOrStr<'a> {
        match self {
            RopeOrStr::Rope(r) => RopeOrStr::Rope(r.slice(crange)),
            RopeOrStr::Str(s) => {
                if crange.end <= crange.start {
                    return RopeOrStr::Str("");
                };
                let mut ci = s.char_indices();
                let (mut cc, mut bs) = (0, None);
                while let Some((i, _)) = ci.next() {
                    if cc == crange.start {
                        cc += 1;
                        bs = Some(i);
                        break;
                    }
                    cc += 1;
                }
                let bs = bs.expect("slice indices out of range");
                while let Some((i, _)) = ci.next() {
                    if cc == crange.start {
                        return RopeOrStr::Str(&s[bs..i]);
                    }
                    cc += 1;
                }
                RopeOrStr::Str(&s[bs..])
            }
        }
    }

    pub(crate) fn graphemes(&self) -> RopeOrStrGraphemes<'a> {
        match self {
            RopeOrStr::Rope(r) => RopeOrStrGraphemes::Rope(RopeGraphemes::new(r)),
            RopeOrStr::Str(s) => RopeOrStrGraphemes::Str(s.graphemes(true)),
        }
    }
}

pub(crate) enum RopeOrStrChars<'a> {
    Rope(RChars<'a>),
    Str(SChars<'a>),
}

impl<'a> Iterator for RopeOrStrChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self {
            RopeOrStrChars::Rope(r) => r.next(),
            RopeOrStrChars::Str(s) => s.next(),
        }
    }
}

pub(crate) enum RopeOrStrGraphemes<'a> {
    Rope(RopeGraphemes<'a>),
    Str(Graphemes<'a>),
}

impl<'a> Iterator for RopeOrStrGraphemes<'a> {
    type Item = RopeOrStr<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RopeOrStrGraphemes::Rope(r) => r.next().map(|r| RopeOrStr::Rope(r)),
            RopeOrStrGraphemes::Str(s) => s.next().map(|s| RopeOrStr::Str(s)),
        }
    }
}
