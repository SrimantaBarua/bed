// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::Range;

use ropey::Rope;

fn range_with<M, V>(rope: &Rope, char_idx: usize, is_match: M, is_valid: V) -> Option<Range<usize>>
where
    M: Fn(char) -> bool,
    V: Fn(char) -> bool,
{
    assert!(char_idx <= rope.len_chars());
    let mut chars = rope.chars_at(char_idx);
    let mut back_chars = chars.clone();
    let end = match chars.next() {
        Some(c) if !is_match(c) => return None,
        Some(_) => {
            let mut cidx = char_idx + 1;
            while let Some(c) = chars.next() {
                if !is_valid(c) {
                    break;
                }
                cidx += 1;
            }
            cidx
        }
        None => return None,
    };
    let mut start = char_idx;
    while let Some(c) = back_chars.prev() {
        if !is_valid(c) {
            break;
        }
        start -= 1;
    }
    Some(Range { start, end })
}

pub(super) fn word_containing(
    rope: &Rope,
    char_idx: usize,
    extended: bool,
) -> Option<Range<usize>> {
    let first = rope.chars_at(char_idx).next();
    range_with(
        rope,
        char_idx,
        |c| !c.is_whitespace(),
        |c| {
            if c.is_whitespace() {
                false
            } else if extended {
                true
            } else {
                match first {
                    // don't have to check for whitespace, since if first is whitespace, we'd be
                    // returning None anyway
                    Some(f) if f != '_' && !f.is_alphanumeric() => false,
                    _ => c == '_' || c.is_alphanumeric(),
                }
            }
        },
    )
}

pub(super) fn space_containing(rope: &Rope, char_idx: usize) -> Option<Range<usize>> {
    range_with(rope, char_idx, |c| c.is_whitespace(), |c| c.is_whitespace())
}
