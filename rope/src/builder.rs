use std::io::{Error as IOError, ErrorKind as IOErrorKind, Read, Result as IOResult};

use super::cow_box::CowBox;
use super::node::{Node, MAX_NODE_SIZE};
use super::Rope;

/// A helper type for building a rope from a reader. This first chunks the input data into a linear
/// vector. Then the `build` method consumes the `RopeBuilder` and returns a `Rope` with the data.
pub(crate) struct RopeBuilder {
    chunks: Vec<String>,
}

impl RopeBuilder {
    /// Create a `RopeBuilder` by reading all the data from a `reader`. The data should be valid
    /// UTF-8.
    pub(crate) fn from_reader<R: Read>(mut reader: R) -> IOResult<RopeBuilder> {
        let mut buffer = vec![0; MAX_NODE_SIZE].into_boxed_slice();
        let mut buffer_length = 0;
        let mut chunks = Vec::new();
        loop {
            let nread = reader.read(&mut buffer[buffer_length..])?;
            if nread == 0 {
                break;
            }
            buffer_length += nread;
            let utf8_end = utf8::last_utf8_boundary(&buffer[..buffer_length]);
            if utf8_end == 0 {
                continue;
            }
            chunks.push(String::from_utf8(buffer[..utf8_end].to_vec()).map_err(|e| {
                IOError::new(
                    IOErrorKind::InvalidData,
                    format!("could not convert to UTF-8: {}", e),
                )
            })?);
            if utf8_end != buffer_length {
                let left_over = buffer_length - utf8_end;
                let (a, b) = buffer[..buffer_length].split_at_mut(utf8_end);
                a[..left_over].copy_from_slice(b);
                buffer_length = left_over;
            } else {
                buffer_length = 0;
            }
        }
        Ok(RopeBuilder { chunks })
    }

    /// Consumes the `RopeBuilder` to return a `Rope` with the owned data.
    pub(crate) fn build(self) -> Rope {
        if self.chunks.is_empty() {
            return Rope::new();
        }
        let mut stack = Vec::new();
        let mut backup_stack = Vec::new();
        for chunk in self.chunks {
            stack.push(Node::new_leaf(chunk));
        }
        while stack.len() > 1 {
            while stack.len() > 1 {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                backup_stack.push(Node::new_inner(left, right));
            }
            while let Some(node) = backup_stack.pop() {
                stack.push(node);
            }
            backup_stack.clear();
        }
        Rope {
            root: CowBox::new(stack.remove(0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    fn open_file(path: &str) -> File {
        File::open(env!("CARGO_MANIFEST_DIR").to_owned() + path).unwrap()
    }

    #[test]
    fn ascii_small() {
        let mut buf = String::new();
        let builder = RopeBuilder::from_reader(open_file("/res/test1.txt")).unwrap();
        assert_eq!(builder.chunks.len(), 1);
        open_file("/res/test1.txt")
            .read_to_string(&mut buf)
            .unwrap();
        assert_eq!(builder.chunks[0], buf);
        assert_eq!(
            builder.build(),
            Rope {
                root: CowBox::new(Node::new_leaf(buf))
            }
        );
    }

    #[test]
    fn ascii_split() {
        let mut buf = String::new();
        let builder = RopeBuilder::from_reader(open_file("/res/test2.txt")).unwrap();
        assert_eq!(builder.chunks.len(), 2);
        open_file("/res/test2.txt")
            .read_to_string(&mut buf)
            .unwrap();
        assert_eq!(builder.chunks[0], &buf[..MAX_NODE_SIZE]);
        assert_eq!(builder.chunks[1], &buf[MAX_NODE_SIZE..]);
        assert_eq!(
            builder.build(),
            Rope {
                root: CowBox::new(Node::new_inner(
                    Node::new_leaf(buf[..MAX_NODE_SIZE].to_owned()),
                    Node::new_leaf(buf[MAX_NODE_SIZE..].to_owned())
                ))
            }
        );
    }

    #[test]
    fn utf8() {
        let mut buf = String::new();
        let builder = RopeBuilder::from_reader(open_file("/res/test3.txt")).unwrap();
        assert_eq!(builder.chunks.len(), 4);
        open_file("/res/test3.txt")
            .read_to_string(&mut buf)
            .unwrap();
        assert_eq!(builder.chunks[0], &buf[..MAX_NODE_SIZE]);
        assert_eq!(builder.chunks[1], &buf[MAX_NODE_SIZE..MAX_NODE_SIZE * 2]);
        assert_eq!(
            builder.chunks[2],
            &buf[MAX_NODE_SIZE * 2..MAX_NODE_SIZE * 3 - 2]
        );
        assert_eq!(builder.chunks[3], &buf[MAX_NODE_SIZE * 3 - 2..]);
        assert_eq!(
            builder.build(),
            Rope {
                root: CowBox::new(Node::new_inner(
                    Node::new_inner(
                        Node::new_leaf(buf[..MAX_NODE_SIZE].to_owned()),
                        Node::new_leaf(buf[MAX_NODE_SIZE..MAX_NODE_SIZE * 2].to_owned())
                    ),
                    Node::new_inner(
                        Node::new_leaf(buf[MAX_NODE_SIZE * 2..MAX_NODE_SIZE * 3 - 2].to_owned()),
                        Node::new_leaf(buf[MAX_NODE_SIZE * 3 - 2..].to_owned())
                    ),
                ))
            }
        );
    }

    #[test]
    #[should_panic(expected = "could not convert to UTF-8")]
    fn fail_utf16() {
        RopeBuilder::from_reader(open_file("/res/test4.txt")).unwrap();
    }
}
