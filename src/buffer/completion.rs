// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fs::read_dir;
use std::path::Path;

use ropey::Rope;

pub(super) enum CompletionSource {
    Path,
}

impl CompletionSource {
    pub(super) fn complete(&self, data: &Rope, offset: usize, list: &mut Vec<String>) {
        match self {
            CompletionSource::Path => self.complete_path(data, offset, list),
        }
    }

    fn complete_path(&self, data: &Rope, offset: usize, list: &mut Vec<String>) {
        // Heuristics. TODO: Improve
        let mut chars = data.chars_at(offset);
        let mut is_dir_start = false;
        match chars.prev() {
            Some('/') => is_dir_start = true,
            None => return,
            _ => {}
        }
        let mut start_off = offset - 1;
        while let Some(c) = chars.prev() {
            if c == '\'' || c == '"' || c.is_whitespace() {
                break;
            }
            start_off -= 1;
        }
        let string_between = format!("{}", data.slice(start_off..offset));
        let abspath = crate::common::abspath(&string_between);
        let path = Path::new(&abspath);
        let base = if is_dir_start {
            ""
        } else {
            if let Some(s) = path.file_name().and_then(|os| os.to_str()) {
                s
            } else {
                return;
            }
        };
        let parent = if is_dir_start {
            if let Some(s) = path.to_str() {
                s
            } else {
                return;
            }
        } else {
            if let Some(s) = path.parent().and_then(|p| p.to_str()) {
                s
            } else {
                return;
            }
        };
        if let Ok(contents) = read_dir(parent) {
            for dirent in contents {
                if let Ok(dirent) = dirent {
                    if let Ok(mut string) = dirent.file_name().into_string() {
                        if let Ok(typ) = dirent.file_type() {
                            if string.starts_with(base) {
                                if typ.is_dir() {
                                    string.push('/');
                                }
                                list.push(string);
                            }
                        }
                    }
                }
            }
        }
    }
}
