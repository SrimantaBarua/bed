// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::Ordering;
use std::fs::read_dir;
use std::path::Path;

use ropey::Rope;

use crate::completion_popup::CompletionOption;
use crate::config::Config;
use crate::theme::Theme;

pub(super) enum CompletionSource {
    Path,
}

impl CompletionSource {
    pub(super) fn complete(
        &self,
        data: &Rope,
        offset: usize,
        config: &Config,
        theme: &Theme,
    ) -> Option<(usize, Vec<CompletionOption>)> {
        match self {
            CompletionSource::Path => self.complete_path(data, offset, config, theme),
        }
    }

    fn complete_path(
        &self,
        data: &Rope,
        offset: usize,
        config: &Config,
        theme: &Theme,
    ) -> Option<(usize, Vec<CompletionOption>)> {
        let mut list = Vec::new();
        // Heuristics. TODO: Improve
        let mut chars = data.chars_at(offset);
        let mut is_dir_start = false;
        match chars.prev() {
            Some('/') => is_dir_start = true,
            None => return None,
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
                return None;
            }
        };
        let parent = if is_dir_start {
            if let Some(s) = path.to_str() {
                s
            } else {
                return None;
            }
        } else {
            if let Some(s) = path.parent().and_then(|p| p.to_str()) {
                s
            } else {
                return None;
            }
        };
        let base_len = base.chars().count();
        let compl_start = if base_len > offset {
            0
        } else {
            offset - base_len
        };
        if let Ok(contents) = read_dir(parent) {
            for dirent in contents {
                if let Ok(dirent) = dirent {
                    if let Ok(mut string) = dirent.file_name().into_string() {
                        if let Ok(typ) = dirent.file_type() {
                            if string.starts_with(base) {
                                let (annotation, color) = if typ.is_dir() {
                                    string.push('/');
                                    (
                                        config.completion_annotation.path_directory.to_owned(),
                                        theme.completion.path_directory,
                                    )
                                } else {
                                    (
                                        config.completion_annotation.path_file.to_owned(),
                                        theme.completion.path_file,
                                    )
                                };
                                list.push(CompletionOption::new(string, annotation, color));
                            }
                        }
                    }
                }
            }
        }
        list.retain(|x| x.option.len() > 0);
        list.sort_by(|a, b| {
            if a.option.chars().next_back() == Some('/') {
                if b.option.chars().next_back() == Some('/') {
                    a.option.cmp(&b.option)
                } else {
                    Ordering::Less
                }
            } else if b.option.chars().next_back() == Some('/') {
                Ordering::Greater
            } else {
                a.option.cmp(&b.option)
            }
        });
        if list.len() > 1 || (list.len() > 0 && list[0].option.len() > offset - compl_start) {
            Some((compl_start, list))
        } else {
            None
        }
    }
}
