// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::Range;

use crate::style::{Color, TextStyle};

// All indices here are codepoint indices
#[derive(Debug)]
pub(super) struct StyledText {
    pub(super) styles: Vec<(usize, TextStyle)>,
    pub(super) colors: Vec<(usize, Color)>,
    pub(super) unders: Vec<(usize, Option<Color>)>,
}

impl StyledText {
    pub(super) fn new(
        len: usize,
        style: TextStyle,
        color: Color,
        under: Option<Color>,
    ) -> StyledText {
        StyledText {
            styles: vec![(len, style)],
            colors: vec![(len, color)],
            unders: vec![(len, under)],
        }
    }

    // TODO: Cut down on code duplication
    pub(super) fn set(
        &mut self,
        range: Range<usize>,
        style: TextStyle,
        color: Color,
        under: Option<Color>,
    ) {
        self.set_style(range.clone(), style);
        self.set_color(range.clone(), color);
        self.set_under(range, under);
    }

    // TODO: Merge adjacent if required
    fn set_style(&mut self, range: Range<usize>, style: TextStyle) {
        let mut i = match self.styles.binary_search_by_key(&range.start, |x| x.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        assert!(i < self.styles.len());
        if range.start == 0 || (i > 0 && range.start == self.styles[i - 1].0) {
            self.styles.insert(i, (range.end, style));
            i += 1;
        } else {
            self.styles.insert(i, (range.start, self.styles[i].1));
            self.styles.insert(i + 1, (range.end, style));
            i += 2;
        }
        // Remove everything after this that is completely covered
        while i < self.styles.len() && self.styles[i].0 <= range.end {
            self.styles.remove(i);
        }
    }

    fn set_color(&mut self, range: Range<usize>, color: Color) {
        assert!(self.colors.len() > 0 && range.start < self.colors[self.colors.len() - 1].0);
        let mut i = match self.colors.binary_search_by_key(&range.start, |x| x.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        assert!(i < self.colors.len());
        if range.start == 0 || (i > 0 && range.start == self.colors[i - 1].0) {
            self.colors.insert(i, (range.end, color));
            i += 1;
        } else {
            self.colors.insert(i, (range.start, self.colors[i].1));
            self.colors.insert(i + 1, (range.end, color));
            i += 2;
        }
        // Remove everything after this that is completely covered
        while i < self.colors.len() && self.colors[i].0 <= range.end {
            self.colors.remove(i);
        }
    }

    fn set_under(&mut self, range: Range<usize>, under: Option<Color>) {
        let mut i = match self.unders.binary_search_by_key(&range.start, |x| x.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        assert!(i < self.unders.len());
        if range.start == 0 || (i > 0 && range.start == self.unders[i - 1].0) {
            self.unders.insert(i, (range.end, under));
            i += 1;
        } else {
            self.unders.insert(i, (range.start, self.unders[i].1));
            self.unders.insert(i + 1, (range.end, under));
            i += 2;
        }
        // Remove everything after this that is completely covered
        while i < self.unders.len() && self.unders[i].0 <= range.end {
            self.unders.remove(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
