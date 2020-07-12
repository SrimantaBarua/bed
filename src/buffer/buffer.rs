// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fs::File;
use std::io::Result as IOResult;

use ropey::Rope;

pub(crate) struct Buffer {
    rope: Rope,
}

impl Buffer {
    pub(super) fn empty() -> Buffer {
        Buffer { rope: Rope::new() }
    }

    pub(super) fn from_file(path: &str) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| Buffer { rope })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| {
                self.rope = rope;
            })
    }
}
