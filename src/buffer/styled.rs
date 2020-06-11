// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::max;
use std::ops::Range;

use crate::style::{Color, TextStyle};

// All indices here are codepoint indices
#[derive(Debug)]
pub(super) struct StyledText {
    pub(crate) indent_depth: usize,
    pub(super) styles: Vec<(usize, TextStyle)>,
    pub(super) colors: Vec<(usize, Color)>,
    pub(super) unders: Vec<(usize, Option<Color>)>,
}

impl StyledText {
    pub(super) fn new(
        len: usize,
        indent_depth: usize,
        style: TextStyle,
        color: Color,
        under: Option<Color>,
    ) -> StyledText {
        StyledText {
            indent_depth,
            styles: vec![(len, style)],
            colors: vec![(len, color)],
            unders: vec![(len, under)],
        }
    }

    pub(super) fn set(
        &mut self,
        range: Range<usize>,
        style: TextStyle,
        color: Color,
        under: Option<Color>,
    ) {
        set(&mut self.styles, range.clone(), style);
        set(&mut self.colors, range.clone(), color);
        set(&mut self.unders, range, under);
    }
}

fn set<T>(vec: &mut Vec<(usize, T)>, range: Range<usize>, val: T)
where
    T: Copy + Eq,
{
    let i = match vec.binary_search_by_key(&range.start, |x| x.0) {
        Ok(i) => i + 1,
        Err(i) => i,
    };
    assert!(i < vec.len());
    while i + 1 < vec.len() && vec[i + 1].0 <= range.end {
        vec.remove(i + 1);
    }
    if (range.start == 0 || (i > 0 && range.start == vec[i - 1].0)) && vec[i].0 <= range.end {
        vec.remove(i);
    }
    if i < vec.len() && val == vec[i].1 {
        vec[i].0 = max(range.end, vec[i].0);
        // Merge with previous if required, and return
        if i > 0 && vec[i - 1].1 == vec[i].1 {
            vec[i - 1].0 = vec[i].0;
            vec.remove(i);
        }
        return;
    }
    if range.start == 0 || (i > 0 && range.start == vec[i - 1].0) {
        if i > 0 && vec[i - 1].1 == val {
            vec[i - 1].0 = range.end;
        } else {
            vec.insert(i, (range.end, val));
        }
        return;
    }
    vec.insert(i, (range.start, vec[i].1));
    if vec[i + 1].0 <= range.end {
        vec[i + 1] = (range.end, val);
    } else {
        vec.insert(i + 1, (range.end, val));
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
