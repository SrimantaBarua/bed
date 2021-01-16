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
pub(crate) type ConcealRanges = RangeTree<bool>;
pub(crate) type ScaleRanges = RangeTree<f64>;

pub(crate) struct StyleRanges {
    style: TextStyleRanges,
    color: ColorRanges,
    under: UnderlineRanges,
    conceal: ConcealRanges,
    scale: ScaleRanges,
}

impl StyleRanges {
    pub(crate) fn new() -> StyleRanges {
        StyleRanges {
            style: TextStyleRanges::new(),
            color: ColorRanges::new(),
            under: UnderlineRanges::new(),
            conceal: ConcealRanges::new(),
            scale: ScaleRanges::new(),
        }
    }

    pub(crate) fn insert_default(&mut self, point: usize, len: usize, color: Color) {
        if len > 0 {
            self.style.insert(point..point, len, TextStyle::default());
            self.color.insert(point..point, len, color);
            self.under.insert(point..point, len, false);
            self.conceal.insert(point..point, len, false);
            self.scale.insert(point..point, len, 1.0);
        }
    }

    pub(crate) fn set_default(&mut self, range: Range<usize>, color: Color) {
        let len = range.len();
        if len > 0 {
            self.style.insert(range.clone(), len, TextStyle::default());
            self.color.insert(range.clone(), len, color);
            self.under.insert(range.clone(), len, false);
            self.conceal.insert(range.clone(), len, false);
            self.scale.insert(range, len, 1.0);
        }
    }
    pub(crate) fn remove(&mut self, range: Range<usize>) {
        if !range.is_empty() {
            self.style.remove(range.clone());
            self.color.remove(range.clone());
            self.under.remove(range.clone());
            self.conceal.remove(range.clone());
            self.scale.remove(range);
        }
    }

    pub(crate) fn set_style(&mut self, range: Range<usize>, style: TextStyle) {
        let len = range.len();
        if len > 0 {
            self.style.insert(range, len, style);
        }
    }

    pub(crate) fn set_color(&mut self, range: Range<usize>, color: Color) {
        let len = range.len();
        if len > 0 {
            self.color.insert(range, len, color);
        }
    }

    pub(crate) fn set_under(&mut self, range: Range<usize>, under: bool) {
        let len = range.len();
        if len > 0 {
            self.under.insert(range, len, under);
        }
    }

    pub(crate) fn set_conceal(&mut self, range: Range<usize>, conceal: bool) {
        let len = range.len();
        if len > 0 {
            self.conceal.insert(range, len, conceal);
        }
    }

    pub(crate) fn set_scale(&mut self, range: Range<usize>, scale: f64) {
        let len = range.len();
        if len > 0 {
            self.scale.insert(range, len, scale);
        }
    }

    pub(crate) fn sub_range<S: SliceRange>(&self, range: Range<usize>) -> StyleSubRanges<S> {
        assert!(!range.is_empty());
        let styles = self.style.iter_range(range.clone()).unwrap().peekable();
        let colors = self.color.iter_range(range.clone()).unwrap().peekable();
        let unders = self.under.iter_range(range.clone()).unwrap().peekable();
        let conceals = self.conceal.iter_range(range.clone()).unwrap().peekable();
        let scales = self.scale.iter_range(range.clone()).unwrap().peekable();
        StyleSubRanges {
            styles,
            colors,
            unders,
            conceals,
            scales,
            phantom: PhantomData,
            offset: range.start,
            cur_start: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StyleSubRanges<'a, S: SliceRange> {
    styles: Peekable<RangeTreeIter<'a, TextStyle>>,
    colors: Peekable<RangeTreeIter<'a, Color>>,
    unders: Peekable<RangeTreeIter<'a, bool>>,
    conceals: Peekable<RangeTreeIter<'a, bool>>,
    scales: Peekable<RangeTreeIter<'a, f64>>,
    phantom: PhantomData<S>,
    offset: usize,
    cur_start: usize,
}

impl<'a, S: SliceRange> Iterator for StyleSubRanges<'a, S> {
    type Item = (S, TextStyle, Color, bool, bool, f64);

    fn next(&mut self) -> Option<(S, TextStyle, Color, bool, bool, f64)> {
        if let Some((style_range, style)) = self.styles.peek() {
            let (color_range, color) = self.colors.peek().unwrap();
            let (under_range, under) = self.unders.peek().unwrap();
            let (conceal_range, conceal) = self.conceals.peek().unwrap();
            let (scale_range, scale) = self.scales.peek().unwrap();
            let min_end = min(
                style_range.end,
                min(
                    color_range.end,
                    min(under_range.end, min(conceal_range.end, scale_range.end)),
                ),
            );
            let range = self.cur_start..min_end - self.offset;
            self.cur_start = min_end - self.offset;
            let ret = (
                S::from_raw(range),
                **style,
                **color,
                **under,
                **conceal,
                **scale,
            );
            if style_range.end == min_end {
                self.styles.next();
            }
            if color_range.end == min_end {
                self.colors.next();
            }
            if under_range.end == min_end {
                self.unders.next();
            }
            if conceal_range.end == min_end {
                self.conceals.next();
            }
            if scale_range.end == min_end {
                self.scales.next();
            }
            Some(ret)
        } else {
            None
        }
    }
}
