// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::min;
use std::iter::Peekable;
use std::marker::PhantomData;
use std::ops::Range;

use crate::common::SliceRange;
use crate::ds::{RangeTree, RangeTreeIter};

mod color;
mod text;

pub(crate) use color::Color;
pub(crate) use text::{TextSize, TextSlant, TextStyle, TextWeight};

pub(crate) type TextStyleRanges = RangeTree<TextStyle>;
pub(crate) type ColorRanges = RangeTree<Color>;
pub(crate) type UnderlineRanges = RangeTree<bool>;

pub(crate) struct StyleRanges {
    pub(crate) style: TextStyleRanges,
    pub(crate) color: ColorRanges,
    pub(crate) under: UnderlineRanges,
}

impl StyleRanges {
    pub(crate) fn new() -> StyleRanges {
        StyleRanges {
            style: TextStyleRanges::new(),
            color: ColorRanges::new(),
            under: UnderlineRanges::new(),
        }
    }

    pub(crate) fn insert_default(&mut self, point: usize, len: usize, color: Color) {
        self.style.insert(point..point, len, TextStyle::default());
        self.color.insert(point..point, len, color);
        self.under.insert(point..point, len, false);
    }

    pub(crate) fn set_default(&mut self, range: Range<usize>, color: Color) {
        let len = range.len();
        self.style.insert(range.clone(), len, TextStyle::default());
        self.color.insert(range.clone(), len, color);
        self.under.insert(range, len, false);
    }
    pub(crate) fn remove(&mut self, range: Range<usize>) {
        self.style.remove(range.clone());
        self.color.remove(range.clone());
        self.under.remove(range);
    }

    pub(crate) fn set_style(&mut self, range: Range<usize>, style: TextStyle) {
        let len = range.len();
        self.style.insert(range, len, style);
    }

    pub(crate) fn set_color(&mut self, range: Range<usize>, color: Color) {
        let len = range.len();
        self.color.insert(range, len, color);
    }

    pub(crate) fn set_under(&mut self, range: Range<usize>, under: bool) {
        let len = range.len();
        self.under.insert(range, len, under);
    }

    pub(crate) fn sub_range<S: SliceRange>(&self, range: Range<usize>) -> StyleSubRanges<S> {
        assert!(!range.is_empty());
        let styles = self.style.iter_range(range.clone()).unwrap().peekable();
        let colors = self.color.iter_range(range.clone()).unwrap().peekable();
        let unders = self.under.iter_range(range.clone()).unwrap().peekable();
        StyleSubRanges {
            styles,
            colors,
            unders,
            phantom: PhantomData,
            offset: range.start,
            cur_start: 0,
        }
    }
}

#[derive(Clone)]
pub(crate) struct StyleSubRanges<'a, S: SliceRange> {
    styles: Peekable<RangeTreeIter<'a, TextStyle>>,
    colors: Peekable<RangeTreeIter<'a, Color>>,
    unders: Peekable<RangeTreeIter<'a, bool>>,
    phantom: PhantomData<S>,
    offset: usize,
    cur_start: usize,
}

impl<'a, S: SliceRange> Iterator for StyleSubRanges<'a, S> {
    type Item = (S, TextStyle, Color, bool);

    fn next(&mut self) -> Option<(S, TextStyle, Color, bool)> {
        if let Some((style_range, style)) = self.styles.peek() {
            let (color_range, color) = self.colors.peek().unwrap();
            let (under_range, under) = self.unders.peek().unwrap();
            let min_end = min(style_range.end, min(color_range.end, under_range.end));
            let range = self.cur_start..min_end - self.offset;
            self.cur_start = min_end - self.offset;
            let ret = (S::from_raw(range), **style, **color, **under);
            if style_range.end == min_end {
                self.styles.next();
            }
            if color_range.end == min_end {
                self.colors.next();
            }
            if under_range.end == min_end {
                self.unders.next();
            }
            Some(ret)
        } else {
            None
        }
    }
}
