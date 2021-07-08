pub mod range {

    use std::ops::Range;

    pub struct SplitRanges {
        remaining: Range<usize>,
        size: usize,
    }

    impl Iterator for SplitRanges {
        type Item = Range<usize>;

        fn next(&mut self) -> Option<Range<usize>> {
            if self.remaining.is_empty() {
                None
            } else if self.remaining.len() <= self.size {
                let ret = self.remaining.clone();
                self.remaining = 0..0;
                Some(ret)
            } else {
                let ret = self.remaining.start..self.remaining.start + self.size;
                self.remaining.start = ret.end;
                Some(ret)
            }
        }
    }

    pub fn split(range: Range<usize>, size: usize) -> SplitRanges {
        SplitRanges {
            remaining: range,
            size,
        }
    }
}
