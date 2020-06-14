// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

// FIXME: This is not a validating URI parser. That could (probably will) cause vulnerabilities.
// Maybe fix them. This is also not a complete URI parser. It doesn't parse the ?query and #frament
// parts of a URI, since I think they're not needed for the language server protocol

use std::convert::TryFrom;
use std::fmt::Write;

use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

#[serde(try_from = "&str")]
#[derive(Clone, Debug, Deserialize)]
pub(in crate::langserver) struct Uri {
    content: String,
    scheme: usize,
    authority: usize,
    path: usize,
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut res = String::new();
        percentage_encode(&self.scheme(), &mut res);
        res.push_str("://");
        if let Some(authority) = self.authority() {
            percentage_encode(authority, &mut res);
        }
        percentage_encode(&self.path(), &mut res);
        serializer.serialize_str(&res)
    }
}

impl Uri {
    pub(in crate::langserver) fn new(s: &str) -> Result<Uri, String> {
        Uri::parse(s)
    }

    pub(in crate::langserver) fn scheme(&self) -> &str {
        &self.content[..self.scheme]
    }

    pub(in crate::langserver) fn authority(&self) -> Option<&str> {
        if self.has_authority() {
            assert!(self.authority >= self.scheme + 3);
            Some(&self.content[self.scheme + 3..self.authority])
        } else {
            assert!(self.authority == self.scheme + 1);
            None
        }
    }

    pub(in crate::langserver) fn path(&self) -> &str {
        &self.content[self.authority..self.path]
    }

    fn has_authority(&self) -> bool {
        self.authority > self.scheme + 1
    }

    fn parse(s: &str) -> Result<Uri, String> {
        if s.len() == 0 {
            return Err("incomplete URI: ".to_owned() + s);
        }
        let bytes = s.as_bytes();
        if !bytes[0].is_ascii_alphabetic() {
            return Err("invalid URI: ".to_owned() + s);
        }
        let mut off = 1;

        // Parse scheme
        while off < bytes.len() {
            match bytes[off] {
                b'+' | b'-' | b'.' => off += 1,
                b':' => break,
                b if b.is_ascii_alphanumeric() => off += 1,
                _ => return Err("invalid URI: ".to_owned() + s),
            }
        }
        let mut content = s[..off].as_bytes().to_owned();
        let scheme = content.len();
        content.push(b':');
        off += 1;

        // Parse optional authority
        if bytes.len() >= off + 2 && bytes[off] == b'/' && bytes[off + 1] == b'/' {
            content.push(b'/');
            content.push(b'/');
            off += 2;
            let len = bytes[off..]
                .iter()
                .position(|&b| b == b'/' || b == b'?' || b == b'#')
                .unwrap_or(bytes[off..].len());
            percentage_decode(&s[off..off + len], &mut content)?;
            off += len;
        }
        let authority = content.len();

        // Insert '/' if required
        if off == scheme + 1
            && off < bytes.len()
            && bytes[off] != b'/'
            && bytes[off] != b'?'
            && bytes[off] != b'#'
        {
            content.push(b'/');
        }

        // Parse path
        let path_len = bytes[off..]
            .iter()
            .position(|&b| b == b'?' || b == b'#')
            .unwrap_or(bytes[off..].len());
        percentage_decode(&s[off..off + path_len], &mut content)?;
        //off += path_len;
        let path = content.len();

        String::from_utf8(content)
            .map(|content| Uri {
                scheme,
                content,
                authority,
                path,
            })
            .map_err(|e| e.to_string())
    }
}

impl TryFrom<&str> for Uri {
    type Error = String;

    fn try_from(s: &str) -> Result<Uri, Self::Error> {
        Uri::parse(s)
    }
}

fn percentage_decode(s: &str, decoded: &mut Vec<u8>) -> Result<(), String> {
    let mut bytes = s.bytes();
    let mut i = 0;
    while let Some(b) = bytes.next() {
        match b {
            b'%' => {
                if s.len() < i + 3 {
                    return Err("incomplete URI".to_owned());
                } else {
                    let u = u8::from_str_radix(&s[i + 1..i + 3], 16)
                        .map_err(|e| format!("{}: {}", e, &s[i + 1..i + 3]))?;
                    i += 3;
                    decoded.push(u);
                    bytes.next();
                    bytes.next();
                }
            }
            //b'-' | b'.' | b'_' | b'~' => decoded.push(b),
            b => decoded.push(b),
        }
        i += 1;
    }
    Ok(())
}

fn percentage_encode(s: &str, res: &mut String) {
    for b in s.bytes() {
        match b {
            b':' => res.push_str("%3A"),
            // b'/' => res.push_str("%2F"),  -- commented out for our specific use case
            b'?' => res.push_str("%3Ff"),
            b'#' => res.push_str("%23"),
            b'[' => res.push_str("%5B"),
            b']' => res.push_str("%5D"),
            b'@' => res.push_str("%40"),
            b'!' => res.push_str("%21"),
            b'$' => res.push_str("%24"),
            b'&' => res.push_str("%26"),
            b'\'' => res.push_str("%27"),
            b'(' => res.push_str("%28"),
            b')' => res.push_str("%29"),
            b'*' => res.push_str("%2A"),
            b'+' => res.push_str("%2B"),
            b',' => res.push_str("%2C"),
            b';' => res.push_str("%2B"),
            b'=' => res.push_str("%3D"),
            b' ' => res.push_str("%20"),
            b'%' => res.push_str("%25"),
            b'-' | b'.' | b'_' | b'~' => res.push(char::from(b)),
            b'/' => res.push('/'), // -- for our specific use case
            b if b.is_ascii_alphanumeric() => res.push(char::from(b)),
            b => {
                res.push('%');
                let _ = write!(res, "{:X}", b);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_no_authority() {
        let uri: Uri = serde_json::from_str("\"file:/home/user/document.txt\"").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert!(uri.authority().is_none());
        assert_eq!(uri.path(), "/home/user/document.txt");

        let uri: Uri = serde_json::from_str("\"file:user%2fdocument.txt\"").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert!(uri.authority().is_none());
        assert_eq!(uri.path(), "/user/document.txt");
    }

    #[test]
    fn decode_with_authority() {
        let uri: Uri = serde_json::from_str("\"file:///home/user/document.txt\"").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert_eq!(uri.authority(), Some(""));
        assert_eq!(uri.path(), "/home/user/document.txt");

        let uri: Uri =
            serde_json::from_str("\"file://authority//home/user/document.txt\"").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert_eq!(uri.authority(), Some("authority"));
        assert_eq!(uri.path(), "//home/user/document.txt");

        let uri: Uri = serde_json::from_str("\"file://authority/user%2fdocument.txt\"").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert_eq!(uri.authority(), Some("authority"));
        assert_eq!(uri.path(), "/user/document.txt");
    }

    #[test]
    fn encode_no_authority() {
        let uri = Uri::new("file:/home/user/document.txt").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert!(uri.authority().is_none());
        assert_eq!(uri.path(), "/home/user/document.txt");

        assert_eq!(
            serde_json::to_string(&uri).unwrap(),
            "\"file:///home/user/document.txt\""
        );

        let uri = Uri::new("file:user/document.txt").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert!(uri.authority().is_none());
        assert_eq!(uri.path(), "/user/document.txt");

        assert_eq!(
            serde_json::to_string(&uri).unwrap(),
            "\"file:///user/document.txt\""
        );
    }

    #[test]
    fn encode_with_authority() {
        let uri = Uri::new("file://authority//home/user/document.txt").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert_eq!(uri.authority(), Some("authority"));
        assert_eq!(uri.path(), "//home/user/document.txt");

        assert_eq!(
            serde_json::to_string(&uri).unwrap(),
            "\"file://authority//home/user/document.txt\""
        );

        let uri = Uri::new("file://authority/user/document.txt").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert_eq!(uri.authority(), Some("authority"));
        assert_eq!(uri.path(), "/user/document.txt");

        assert_eq!(
            serde_json::to_string(&uri).unwrap(),
            "\"file://authority/user/document.txt\""
        );
    }
}
