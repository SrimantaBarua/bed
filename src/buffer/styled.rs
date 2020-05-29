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
        assert_eq!(styled.unders, vec![(1, None), (4, None), (10, None)]);
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
            vec![
                (1, None),
                (4, None),
                (5, None),
                (8, Some(Color::new(0, 0, 0, 0xff))),
                (10, None)
            ]
        );
        assert_eq!(
            styled.styles,
            vec![
                (1, TextStyle::default()),
                (4, TextStyle::new(TextWeight::Bold, TextSlant::Italic)),
                (5, TextStyle::default()),
                (8, TextStyle::default()),
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
                (4, Color::new(0xff, 0, 0, 0)),
                (5, Color::new(0xff, 0, 0, 0)),
                (8, Color::new(0xff, 0, 0, 0)),
                (10, Color::new(0, 0, 0, 0))
            ]
        );
        assert_eq!(
            styled.unders,
            vec![
                (1, None),
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
                (8, TextStyle::default()),
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
                (3, Color::new(0xff, 0, 0, 0)),
                (7, Color::new(0xff, 0, 0, 0)),
                (8, Color::new(0xff, 0, 0, 0)),
                (10, Color::new(0, 0, 0, 0))
            ]
        );
        assert_eq!(
            styled.unders,
            vec![
                (1, None),
                (3, None),
                (7, None),
                (8, Some(Color::new(0, 0, 0, 0xff))),
                (10, None)
            ]
        );
        assert_eq!(
            styled.styles,
            vec![
                (1, TextStyle::default()),
                (3, TextStyle::new(TextWeight::Bold, TextSlant::Italic)),
                (7, TextStyle::new(TextWeight::Light, TextSlant::Oblique)),
                (8, TextStyle::default()),
                (10, TextStyle::default())
            ]
        );
    }
}
