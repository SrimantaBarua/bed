use std::fs::File;
use std::io::Read;
use std::ops::Range;

use rope::Rope;

fn open_file(path: &str) -> File {
    File::open(env!("CARGO_MANIFEST_DIR").to_owned() + path).unwrap()
}

struct StrLines<'a> {
    lines: std::iter::Peekable<std::str::Lines<'a>>,
    has_last_line: bool,
}

impl<'a> StrLines<'a> {
    fn new(s: &'a str) -> StrLines<'a> {
        let has_last_line = s.ends_with('\n');
        StrLines {
            lines: s.lines().peekable(),
            has_last_line,
        }
    }
}

impl<'a> Iterator for StrLines<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if let Some(line) = self.lines.next() {
            if !self.has_last_line && self.lines.peek().is_none() {
                return Some(line.to_owned());
            }
            return Some(line.to_owned() + "\n");
        }
        if self.has_last_line {
            self.has_last_line = false;
            return Some("".to_owned());
        }
        None
    }
}

fn line_to_byte(s: &str, linum: usize) -> usize {
    let mut cur_line = 0;
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if cur_line == linum {
            return i;
        }
        if bytes[i] == b'\n' {
            cur_line += 1;
            if cur_line == linum {
                return i + 1;
            }
        }
    }
    panic!("line index out of bounds");
}

fn byte_to_line(s: &str, bidx: usize) -> usize {
    s.bytes().take(bidx).filter(|b| *b == b'\n').count()
}

fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).unwrap().0
}

fn byte_to_char(s: &str, bidx: usize) -> usize {
    s[..bidx].chars().count()
}

#[test]
fn len_bytes() {
    let mut buf = String::new();
    let mut do_it = |path, range: Range<usize>| {
        let rope = Rope::from_reader(open_file(path)).unwrap();
        open_file(path).read_to_string(&mut buf).unwrap();
        assert_eq!(rope.len_bytes(), buf.len());
        let ropeslice = rope.slice(range.clone());
        let bufslice = &buf[range];
        assert_eq!(ropeslice.len_bytes(), bufslice.len());
        buf.clear();
    };
    do_it("/res/test1.txt", 5..2408);
    do_it("/res/test2.txt", 2000..3094);
    do_it("/res/test3.txt", 5..8014);
}

#[test]
fn len_chars() {
    let mut buf = String::new();
    let mut do_it = |path, range: Range<usize>| {
        let rope = Rope::from_reader(open_file(path)).unwrap();
        open_file(path).read_to_string(&mut buf).unwrap();
        assert_eq!(rope.len_chars(), buf.chars().count());
        let ropeslice = rope.slice(range.clone());
        let bufslice = &buf[range];
        assert_eq!(ropeslice.len_chars(), bufslice.chars().count());
        buf.clear();
    };
    do_it("/res/test1.txt", 5..2408);
    do_it("/res/test2.txt", 2000..3094);
    do_it("/res/test3.txt", 5..8014);
}

#[test]
#[should_panic(expected = "slice index out of bounds")]
fn slice_fail() {
    let rope = Rope::from_reader(open_file("/res/test1.txt")).unwrap();
    assert_eq!(rope.len_bytes(), 2412);
    rope.slice(..2413);
}

#[test]
fn compare_string() {
    let mut buf = String::new();
    let mut do_it = |path| {
        open_file(path).read_to_string(&mut buf).unwrap();
        assert_eq!(Rope::from_reader(open_file(path)).unwrap().to_string(), buf);
        buf.clear();
    };
    do_it("/res/test1.txt");
    do_it("/res/test2.txt");
    do_it("/res/test3.txt");
}

#[test]
fn compare_slice_string() {
    let mut buf = String::new();
    let rope = Rope::from_reader(open_file("/res/test3.txt")).unwrap();
    open_file("/res/test3.txt")
        .read_to_string(&mut buf)
        .unwrap();
    let slice = rope.slice(1000..8002);
    let buf_slice = &buf[1000..8002];
    assert_eq!(&slice.to_string(), buf_slice);
}

#[test]
fn compare_iterators_empty() {
    let rope = Rope::new();
    assert!(rope.chars().eq("".chars()));
    assert!(rope.char_indices().eq("".char_indices()));
    assert!(rope
        .lines()
        .map(|line| line.to_string())
        .eq(StrLines::new("")));
}

#[test]
fn compare_iterators() {
    let mut buf = String::new();
    let mut do_it = |path| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let rope = Rope::from_reader(open_file(path)).unwrap();
        assert!(rope.chars().eq(buf.chars()));
        assert!(rope.char_indices().eq(buf.char_indices()));
        assert!(rope
            .lines()
            .map(|line| line.to_string())
            .eq(StrLines::new(&buf)));
        buf.clear();
    };
    do_it("/res/test1.txt");
    do_it("/res/test2.txt");
    do_it("/res/test3.txt");
}

#[test]
fn insertion_empty() {
    let mut rope = Rope::new();
    assert_eq!(rope.to_string(), "".to_owned());
    rope.insert(0, "====XYZA====");
    assert_eq!(rope.to_string(), "====XYZA====");
    rope.insert_char(4, 'x');
    assert_eq!(rope.to_string(), "====xXYZA====");
}

#[test]
fn insertion() {
    let mut buf = String::new();
    let mut rope = Rope::from_reader(open_file("/res/test3.txt")).unwrap();
    open_file("/res/test3.txt")
        .read_to_string(&mut buf)
        .unwrap();
    assert_eq!(rope.to_string(), buf);
    rope.insert(1000, "====XYZA====");
    buf.insert_str(1000, "====XYZA====");
    assert_eq!(rope.to_string(), buf);
    rope.insert_char(8014, 'x');
    buf.insert(8014, 'x');
    assert_eq!(rope.to_string(), buf);
}

#[test]
fn remove() {
    let mut buf = String::new();
    let mut do_it = |path, range: Range<usize>| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let mut rope = Rope::from_reader(open_file(path)).unwrap();
        buf.replace_range(range.clone(), "");
        rope.remove(range);
        assert!(rope.chars().eq(buf.chars()));
        assert!(rope.char_indices().eq(buf.char_indices()));
        assert!(rope
            .lines()
            .map(|line| line.to_string())
            .eq(StrLines::new(&buf)));
        buf.clear();
    };
    do_it("/res/test1.txt", 350..600);
    do_it("/res/test2.txt", 0..4096);
    do_it("/res/test3.txt", 1000..8002);
}

#[test]
fn len_lines() {
    let mut buf = String::new();
    let mut do_it = |path, range: Range<usize>| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let mut rope = Rope::from_reader(open_file(path)).unwrap();
        let diff = if buf.ends_with('\n') { 1 } else { 0 };
        assert_eq!(rope.len_lines() - diff, buf.lines().count());
        buf.replace_range(range.clone(), "");
        rope.remove(range);
        let diff = if buf.ends_with('\n') { 1 } else { 0 };
        assert_eq!(rope.len_lines() - diff, buf.lines().count());
        buf.clear();
    };
    do_it("/res/test1.txt", 350..600);
    do_it("/res/test2.txt", 0..4096);
    do_it("/res/test3.txt", 1000..8002);
}

#[test]
fn slice_len_lines() {
    let mut buf = String::new();
    let mut do_it = |path, del_range: Range<usize>, slice_range: Range<usize>| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let mut rope = Rope::from_reader(open_file(path)).unwrap();
        let bufslice = &buf[slice_range.clone()];
        let ropeslice = rope.slice(slice_range.clone());
        let diff = if bufslice.ends_with('\n') { 1 } else { 0 };
        assert_eq!(ropeslice.len_lines() - diff, bufslice.lines().count());
        buf.replace_range(del_range.clone(), "");
        rope.remove(del_range);
        let bufslice = &buf[slice_range.clone()];
        let ropeslice = rope.slice(slice_range.clone());
        let diff = if bufslice.ends_with('\n') { 1 } else { 0 };
        assert_eq!(ropeslice.len_lines() - diff, bufslice.lines().count());
        buf.clear();
    };
    do_it("/res/test1.txt", 350..600, 5..200);
    do_it("/res/test2.txt", 0..4096, 0..5);
    do_it("/res/test3.txt", 1000..8002, 5..2006);
}

#[test]
fn line_indices() {
    let mut buf = String::new();
    let mut do_it = |path, range: Range<usize>| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let mut rope = Rope::from_reader(open_file(path)).unwrap();
        assert!((0..rope.len_lines())
            .map(|i| rope.line(i).to_string())
            .eq(StrLines::new(&buf)));
        buf.replace_range(range.clone(), "");
        rope.remove(range);
        assert!((0..rope.len_lines())
            .map(|i| rope.line(i).to_string())
            .eq(StrLines::new(&buf)));
        buf.clear();
    };
    do_it("/res/test1.txt", 350..600);
    do_it("/res/test2.txt", 0..4096);
    do_it("/res/test3.txt", 1000..8002);
}

#[test]
fn slice_line_indices() {
    let mut buf = String::new();
    let mut do_it = |path, del_range: Range<usize>, slice_range: Range<usize>| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let mut rope = Rope::from_reader(open_file(path)).unwrap();
        let bufslice = &buf[slice_range.clone()];
        let ropeslice = rope.slice(slice_range.clone());
        assert!((0..ropeslice.len_lines())
            .map(|i| ropeslice.line(i).to_string())
            .eq(StrLines::new(bufslice)));
        buf.replace_range(del_range.clone(), "");
        rope.remove(del_range);
        let bufslice = &buf[slice_range.clone()];
        let ropeslice = rope.slice(slice_range.clone());
        assert!((0..ropeslice.len_lines())
            .map(|i| ropeslice.line(i).to_string())
            .eq(StrLines::new(bufslice)));
        buf.clear();
    };
    do_it("/res/test1.txt", 350..600, 5..200);
    do_it("/res/test2.txt", 0..4096, 0..5);
    do_it("/res/test3.txt", 1000..8002, 5..2006);
}

#[test]
fn line_byte_indices() {
    let mut buf = String::new();
    let mut do_it = |path, range: Range<usize>, byte_indices: &[usize], line_indices: &[usize]| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let mut rope = Rope::from_reader(open_file(path)).unwrap();
        for &li in line_indices.iter() {
            if li < rope.len_lines() {
                assert_eq!(rope.line_to_byte(li), line_to_byte(&buf, li));
            }
        }
        for &bi in byte_indices.iter() {
            if bi < rope.len_bytes() {
                assert_eq!(rope.byte_to_line(bi), byte_to_line(&buf, bi));
            }
        }
        buf.replace_range(range.clone(), "");
        rope.remove(range);
        for &li in line_indices.iter() {
            if li < rope.len_lines() {
                assert_eq!(rope.line_to_byte(li), line_to_byte(&buf, li));
            }
        }
        for &bi in byte_indices.iter() {
            if bi < rope.len_bytes() {
                assert_eq!(rope.byte_to_line(bi), byte_to_line(&buf, bi));
            }
        }
        buf.clear();
    };
    do_it("/res/test1.txt", 350..600, &[20, 100, 1000], &[2, 10]);
    do_it("/res/test2.txt", 0..4096, &[50, 100], &[5, 10]);
    do_it("/res/test3.txt", 1000..8002, &[300, 4000], &[10, 300, 500]);
}

#[test]
fn slice_line_byte_indices() {
    let mut buf = String::new();
    let mut do_it = |path,
                     del_range: Range<usize>,
                     slice_range: Range<usize>,
                     byte_indices: &[usize],
                     line_indices: &[usize]| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let mut rope = Rope::from_reader(open_file(path)).unwrap();
        let bufslice = &buf[slice_range.clone()];
        let ropeslice = rope.slice(slice_range.clone());
        for &li in line_indices.iter() {
            if li < ropeslice.len_lines() {
                assert_eq!(ropeslice.line_to_byte(li), line_to_byte(&bufslice, li));
            }
        }
        for &bi in byte_indices.iter() {
            if bi < ropeslice.len_bytes() {
                assert_eq!(ropeslice.byte_to_line(bi), byte_to_line(&bufslice, bi));
            }
        }
        buf.replace_range(del_range.clone(), "");
        rope.remove(del_range);
        let bufslice = &buf[slice_range.clone()];
        let ropeslice = rope.slice(slice_range.clone());
        for &li in line_indices.iter() {
            if li < ropeslice.len_lines() {
                assert_eq!(ropeslice.line_to_byte(li), line_to_byte(&bufslice, li));
            }
        }
        for &bi in byte_indices.iter() {
            if bi < ropeslice.len_bytes() {
                assert_eq!(ropeslice.byte_to_line(bi), byte_to_line(&bufslice, bi));
            }
        }
        buf.clear();
    };
    do_it(
        "/res/test1.txt",
        350..600,
        5..200,
        &[20, 100, 1000],
        &[2, 10],
    );
    do_it("/res/test2.txt", 0..4096, 0..5, &[50, 100], &[5, 10]);
    do_it(
        "/res/test3.txt",
        1000..8002,
        6..2006,
        &[20, 1000],
        &[20, 100],
    );
}

#[test]
fn char_byte_indices() {
    let mut buf = String::new();
    let mut do_it = |path, range: Range<usize>, byte_indices: &[usize], char_indices: &[usize]| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let mut rope = Rope::from_reader(open_file(path)).unwrap();
        for &li in char_indices.iter() {
            if li < rope.len_chars() {
                assert_eq!(rope.char_to_byte(li), char_to_byte(&buf, li));
            }
        }
        for &bi in byte_indices.iter() {
            if bi < rope.len_bytes() {
                assert_eq!(rope.byte_to_char(bi), byte_to_char(&buf, bi));
            }
        }
        buf.replace_range(range.clone(), "");
        rope.remove(range);
        for &li in char_indices.iter() {
            if li < rope.len_chars() {
                assert_eq!(rope.char_to_byte(li), char_to_byte(&buf, li));
            }
        }
        for &bi in byte_indices.iter() {
            if bi < rope.len_bytes() {
                assert_eq!(rope.byte_to_char(bi), byte_to_char(&buf, bi));
            }
        }
        buf.clear();
    };
    do_it("/res/test1.txt", 10..20, &[20, 100, 1000], &[2, 10]);
    do_it("/res/test2.txt", 0..4096, &[50, 100], &[5, 10]);
    do_it("/res/test3.txt", 1000..8002, &[300, 4000], &[10, 300, 500]);
}

#[test]
fn slice_char_byte_indices() {
    let mut buf = String::new();
    let mut do_it = |path,
                     del_range: Range<usize>,
                     slice_range: Range<usize>,
                     byte_indices: &[usize],
                     char_indices: &[usize]| {
        open_file(path).read_to_string(&mut buf).unwrap();
        let mut rope = Rope::from_reader(open_file(path)).unwrap();
        let bufslice = &buf[slice_range.clone()];
        let ropeslice = rope.slice(slice_range.clone());
        for &li in char_indices.iter() {
            if li < ropeslice.len_chars() {
                assert_eq!(ropeslice.char_to_byte(li), char_to_byte(&bufslice, li));
            }
        }
        for &bi in byte_indices.iter() {
            if bi < ropeslice.len_bytes() {
                assert_eq!(ropeslice.byte_to_char(bi), byte_to_char(&bufslice, bi));
            }
        }
        buf.replace_range(del_range.clone(), "");
        rope.remove(del_range);
        let bufslice = &buf[slice_range.clone()];
        let ropeslice = rope.slice(slice_range.clone());
        for &li in char_indices.iter() {
            if li < ropeslice.len_chars() {
                assert_eq!(ropeslice.char_to_byte(li), char_to_byte(&bufslice, li));
            }
        }
        for &bi in byte_indices.iter() {
            if bi < ropeslice.len_bytes() {
                assert_eq!(ropeslice.byte_to_char(bi), byte_to_char(&bufslice, bi));
            }
        }
        buf.clear();
    };
    do_it("/res/test1.txt", 10..20, 5..200, &[20, 100, 1000], &[2, 10]);
    do_it("/res/test2.txt", 0..4096, 0..5, &[50, 100], &[5, 10]);
    do_it(
        "/res/test3.txt",
        1000..8002,
        6..2006,
        &[20, 1000],
        &[20, 100],
    );
}
