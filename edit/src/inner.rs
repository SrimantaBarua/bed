use std::io::{Read, Result as IOResult};
use std::ops::{Bound, RangeBounds};

use rope::Rope;
use util::hash::FnvHashMap;

use super::point::Point;
use super::view_state::ViewState;

pub(crate) struct BufferInner {
    rope: Rope,
    view_id: usize,
    views: FnvHashMap<usize, ViewState>,
}

impl BufferInner {
    pub(crate) fn new() -> BufferInner {
        BufferInner {
            rope: Rope::new(),
            view_id: 0,
            views: FnvHashMap::default(),
        }
    }

    pub(crate) fn from_reader<R: Read>(reader: R) -> IOResult<BufferInner> {
        Rope::from_reader(reader).map(|rope| BufferInner {
            rope,
            view_id: 0,
            views: FnvHashMap::default(),
        })
    }

    pub(crate) fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub(crate) fn create_view(&mut self) -> usize {
        let view_id = self.view_id;
        self.view_id += 1;
        self.views.insert(view_id, ViewState::new());
        view_id
    }

    pub(crate) fn delete_view(&mut self, view_id: usize) {
        self.views.remove(&view_id);
    }

    pub(crate) fn contains_point(&self, point: &Point) -> bool {
        if point.line >= self.rope.len_lines() {
            return false;
        }
        let line = self.rope.line(point.line);
        point.char_offset < line.len_chars()
    }

    pub(crate) fn insert_string(&mut self, point: &Point, s: &str) {
        self.rope.insert(self.point_to_offset(point), s);
    }

    pub(crate) fn remove<R>(&mut self, range: R)
    where
        R: RangeBounds<Point>,
    {
        match range.start_bound() {
            Bound::Unbounded => match range.end_bound() {
                Bound::Unbounded => self.rope.remove(..),
                Bound::Included(end) => self.rope.remove(..=self.point_to_offset(end)),
                Bound::Excluded(end) => self.rope.remove(..self.point_to_offset(end)),
            },
            Bound::Included(start) => {
                let start = self.point_to_offset(start);
                match range.end_bound() {
                    Bound::Unbounded => self.rope.remove(..),
                    Bound::Included(end) => self.rope.remove(start..=self.point_to_offset(end)),
                    Bound::Excluded(end) => self.rope.remove(start..self.point_to_offset(end)),
                }
            }
            Bound::Excluded(_) => {
                panic!("do we have ranges with excluded starts?")
            }
        }
    }

    fn point_to_offset(&self, point: &Point) -> usize {
        assert!(self.contains_point(point));
        self.rope.line_to_char(point.line) + point.char_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    fn open_file(path: &str) -> File {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/../rope/res/" + path;
        File::open(path).unwrap()
    }

    #[test]
    fn buffer_contains_point() {
        let bi = BufferInner::from_reader(open_file("test1.txt")).unwrap();
        assert!(bi.contains_point(&Point::new(5, 5)));
        assert!(!bi.contains_point(&Point::new(200, 5)));
        assert!(!bi.contains_point(&Point::new(5, 500)));
    }

    #[test]
    fn buffer_insert() {
        let mut bi = BufferInner::from_reader(open_file("test1.txt")).unwrap();
        assert_eq!(bi.len_chars(), 2412);
        assert_eq!(
            bi.rope.line(5).to_string(),
            "abcdefghijklmnopqrst".repeat(10) + "\n"
        );
        bi.insert_string(&Point::new(5, 5), "XYZA");
        assert_eq!(bi.len_chars(), 2416);
        assert_eq!(
            bi.rope.line(5).to_string(),
            "abcdeXYZAfghijklmnopqrst".to_owned() + &"abcdefghijklmnopqrst".repeat(9) + "\n"
        );
    }
}
