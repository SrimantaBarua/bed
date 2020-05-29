// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::max;
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
        let i = match self.styles.binary_search_by_key(&range.start, |x| x.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        assert!(i < self.styles.len());
        while i + 1 < self.styles.len() && self.styles[i + 1].0 <= range.end {
            self.styles.remove(i + 1);
        }
        if (range.start == 0 || (i > 0 && range.start == self.styles[i - 1].0))
            && self.styles[i].0 <= range.end
        {
            self.styles.remove(i);
        }
        if i < self.styles.len() && style == self.styles[i].1 {
            self.styles[i].0 = max(range.end, self.styles[i].0);
            // Merge with previous if required, and return
            if i > 0 && self.styles[i - 1].1 == self.styles[i].1 {
                self.styles[i - 1].0 = self.styles[i].0;
                self.styles.remove(i);
            }
            return;
        }
        if range.start == 0 || (i > 0 && range.start == self.styles[i - 1].0) {
            if i > 0 && self.styles[i - 1].1 == style {
                self.styles[i - 1].0 = range.end;
            } else {
                self.styles.insert(i, (range.end, style));
            }
            return;
        }
        self.styles.insert(i, (range.start, self.styles[i].1));
        if self.styles[i + 1].0 <= range.end {
            self.styles[i + 1] = (range.end, style);
        } else {
            self.styles.insert(i + 1, (range.end, style));
        }
    }

    fn set_color(&mut self, range: Range<usize>, color: Color) {
        let i = match self.colors.binary_search_by_key(&range.start, |x| x.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        assert!(i < self.colors.len());
        while i + 1 < self.colors.len() && self.colors[i + 1].0 <= range.end {
            self.colors.remove(i + 1);
        }
        if (range.start == 0 || (i > 0 && range.start == self.colors[i - 1].0))
            && self.colors[i].0 <= range.end
        {
            self.colors.remove(i);
        }
        if i < self.colors.len() && color == self.colors[i].1 {
            self.colors[i].0 = max(range.end, self.colors[i].0);
            // Merge with previous if required, and return
            if i > 0 && self.colors[i - 1].1 == self.colors[i].1 {
                self.colors[i - 1].0 = self.colors[i].0;
                self.colors.remove(i);
            }
            return;
        }
        if range.start == 0 || (i > 0 && range.start == self.colors[i - 1].0) {
            if i > 0 && self.colors[i - 1].1 == color {
                self.colors[i - 1].0 = range.end;
            } else {
                self.colors.insert(i, (range.end, color));
            }
            return;
        }
        self.colors.insert(i, (range.start, self.colors[i].1));
        if self.colors[i + 1].0 <= range.end {
            self.colors[i + 1] = (range.end, color);
        } else {
            self.colors.insert(i + 1, (range.end, color));
        }
    }

    fn set_under(&mut self, range: Range<usize>, under: Option<Color>) {
        let i = match self.unders.binary_search_by_key(&range.start, |x| x.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        assert!(i < self.unders.len());
        while i + 1 < self.unders.len() && self.unders[i + 1].0 <= range.end {
            self.unders.remove(i + 1);
        }
        if (range.start == 0 || (i > 0 && range.start == self.unders[i - 1].0))
            && self.unders[i].0 <= range.end
        {
            self.unders.remove(i);
        }
        if i < self.unders.len() && under == self.unders[i].1 {
            self.unders[i].0 = max(range.end, self.unders[i].0);
            // Merge with previous if required, and return
            if i > 0 && self.unders[i - 1].1 == self.unders[i].1 {
                self.unders[i - 1].0 = self.unders[i].0;
                self.unders.remove(i);
            }
            return;
        }
        if range.start == 0 || (i > 0 && range.start == self.unders[i - 1].0) {
            if i > 0 && self.unders[i - 1].1 == under {
                self.unders[i - 1].0 = range.end;
            } else {
                self.unders.insert(i, (range.end, under));
            }
            return;
        }
        self.unders.insert(i, (range.start, self.unders[i].1));
        if self.unders[i + 1].0 <= range.end {
            self.unders[i + 1] = (range.end, under);
        } else {
            self.unders.insert(i + 1, (range.end, under));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::*;

    #[test]
    fn test_new_styled() {
        let styled = StyledText::new(5, TextStyle::default(), Color::new(0, 0, 0, 0), None);
        assert_eq!(styled.colors, vec![(5, Color::new(0, 0, 0, 0))]);
        assert_eq!(styled.unders, vec![(5, None)]);
        assert_eq!(styled.styles, vec![(5, TextStyle::default())]);
    }

    #[test]
    fn test_set() {
        let mut styled = StyledText::new(10, TextStyle::default(), Color::new(0, 0, 0, 0), None);

        styled.set(
            1..4,
            TextStyle::new(TextWeight::Bold, TextSlant::Italic),
            Color::new(0xff, 0, 0, 0),
            None,
        );
        assert_eq!(
            styled.colors,
            vec![
                (1, Color::new(0, 0, 0, 0)),
                (4, Color::new(0xff, 0, 0, 0)),
                (10, Color::new(0, 0, 0, 0))
            ]
        );
        assert_eq!(styled.unders, vec![(10, None)]);
        assert_eq!(
            styled.styles,
            vec![
                (1, TextStyle::default()),
                (4, TextStyle::new(TextWeight::Bold, TextSlant::Italic)),
                (10, TextStyle::default())
            ]
        );

        styled.set(
            5..8,
            TextStyle::default(),
            Color::new(0xff, 0, 0, 0),
            Some(Color::new(0, 0, 0, 0xff)),
        );
        assert_eq!(
            styled.colors,
            vec![
                (1, Color::new(0, 0, 0, 0)),
                (4, Color::new(0xff, 0, 0, 0)),
                (5, Color::new(0, 0, 0, 0)),
                (8, Color::new(0xff, 0, 0, 0)),
                (10, Color::new(0, 0, 0, 0))
            ]
        );
        assert_eq!(
            styled.unders,
            vec![(5, None), (8, Some(Color::new(0, 0, 0, 0xff))), (10, None)]
        );
        assert_eq!(
            styled.styles,
            vec![
                (1, TextStyle::default()),
                (4, TextStyle::new(TextWeight::Bold, TextSlant::Italic)),
                (10, TextStyle::default())
            ]
        );

        styled.set(
            4..5,
            TextStyle::new(TextWeight::Light, TextSlant::Oblique),
            Color::new(0xff, 0, 0, 0),
            Some(Color::new(0xff, 0, 0, 0)),
        );
        assert_eq!(
            styled.colors,
            vec![
                (1, Color::new(0, 0, 0, 0)),
                (8, Color::new(0xff, 0, 0, 0)),
                (10, Color::new(0, 0, 0, 0))
            ]
        );
        assert_eq!(
            styled.unders,
            vec![
                (4, None),
                (5, Some(Color::new(0xff, 0, 0, 0))),
                (8, Some(Color::new(0, 0, 0, 0xff))),
                (10, None)
            ]
        );
        assert_eq!(
            styled.styles,
            vec![
                (1, TextStyle::default()),
                (4, TextStyle::new(TextWeight::Bold, TextSlant::Italic)),
                (5, TextStyle::new(TextWeight::Light, TextSlant::Oblique)),
                (10, TextStyle::default())
            ]
        );

        styled.set(
            3..7,
            TextStyle::new(TextWeight::Light, TextSlant::Oblique),
            Color::new(0xff, 0, 0, 0),
            None,
        );
        assert_eq!(
            styled.colors,
            vec![
                (1, Color::new(0, 0, 0, 0)),
                (8, Color::new(0xff, 0, 0, 0)),
                (10, Color::new(0, 0, 0, 0))
            ]
        );
        assert_eq!(
            styled.unders,
            vec![(7, None), (8, Some(Color::new(0, 0, 0, 0xff))), (10, None)]
        );
        assert_eq!(
            styled.styles,
            vec![
                (1, TextStyle::default()),
                (3, TextStyle::new(TextWeight::Bold, TextSlant::Italic)),
                (7, TextStyle::new(TextWeight::Light, TextSlant::Oblique)),
                (10, TextStyle::default())
            ]
        );
    }
}
