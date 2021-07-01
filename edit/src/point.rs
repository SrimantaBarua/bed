#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub line: usize,
    pub offset: usize,
}

impl Point {
    pub fn new(line: usize, offset: usize) -> Point {
        Point { line, offset }
    }
}
