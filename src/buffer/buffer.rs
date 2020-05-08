// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fs::File;
use std::io::Result as IOResult;

use ropey::Rope;

pub(crate) struct Buffer {
    data: Rope,
}

impl Buffer {
    pub(super) fn empty() -> Buffer {
        Buffer { data: Rope::new() }
    }

    pub(super) fn from_file(path: &str) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| Buffer { data: rope })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| self.data = rope)
    }
}
