/// Get the last index in the slice of bytes that is a valid UTF-8 boundary. i.e., the previous
/// UTF-8 character's last byte was at `(index - 1)`. Of course, the assumption is that the bytes
/// form a valid UTF-8 string. So we don't validate the UTF-8 characters we encounter.
pub fn last_utf8_boundary(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }
    let mut index = bytes.len();
    if is_ascii(bytes[index - 1]) {
        // Previous byte is ASCII, so return the current index.
        return index;
    }
    while index > 0 && is_utf8_continuation(bytes[index - 1]) {
        index -= 1;
    }
    if index == 0 {
        return 0;
    }
    let expected_length = length_from_first_byte(bytes[index - 1]);
    if index + expected_length - 1 == bytes.len() {
        bytes.len()
    } else {
        index - 1
    }
}

fn is_ascii(byte: u8) -> bool {
    byte <= 0x7f
}

fn is_utf8_continuation(byte: u8) -> bool {
    (byte & 0xc0) == 0x80
}

fn length_from_first_byte(byte: u8) -> usize {
    assert!(!is_ascii(byte) && !is_utf8_continuation(byte));
    // UTF-8 five and six byte sequences aren't accepted. The max UTF-8 codepoint is U+10FFFF.
    assert!((byte & 0xfc) != 0xf8);
    TABLE[byte as usize] as usize
}

#[rustfmt::skip]
static TABLE: [u8; 256] = [
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x00 - 0x0f
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x10 - 0x1f
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x20 - 0x2f
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x30 - 0x3f
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x40 - 0x4f
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x50 - 0x5f
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x60 - 0x6f
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x70 - 0x7f
  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x80 - 0x8f
  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x90 - 0x9f
  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0xa0 - 0xaf
  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0xb0 - 0xbf
  2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0xc0 - 0xcf
  2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0xd0 - 0xdf
  3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xe0 - 0xef
  4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, // 0xf0 - 0xff
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_utf8_boundary() {
        let bytes_a = b"abcdefgh";
        let bytes_b = b"abcde\xc2\x82h";
        let bytes_c = b"abcdef\xc2\x82";
        let bytes_d = b"abcdef\xe2\x82";
        let bytes_e = b"abcdefg\xe2";
        assert_eq!(last_utf8_boundary(bytes_a), 8);
        assert_eq!(last_utf8_boundary(bytes_b), 8);
        assert_eq!(last_utf8_boundary(bytes_c), 8);
        assert_eq!(last_utf8_boundary(bytes_d), 6);
        assert_eq!(last_utf8_boundary(bytes_e), 7);
    }
}
