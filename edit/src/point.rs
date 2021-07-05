#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub line: usize,
    pub char_offset: usize,
}

impl Point {
    pub fn new(line: usize, char_offset: usize) -> Point {
        Point { line, char_offset }
    }
}
