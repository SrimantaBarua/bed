// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::default::Default;
use std::fs::read_to_string;
use std::rc::Rc;

use directories::ProjectDirs;
use fnv::FnvHashMap;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::style::{Color, TextSlant, TextWeight};

#[derive(Deserialize)]
pub(crate) struct ThemeTextview {
    pub(crate) background: Color,
    pub(crate) foreground: Color,
    pub(crate) cursor_line: Color,
    pub(crate) cursor: Color,
    pub(crate) border_width: u32,
    pub(crate) border_color: Color,
    pub(crate) indent_guide: Color,
    pub(crate) lint_warnings: Option<Color>,
    pub(crate) lint_errors: Option<Color>,
}

impl Default for ThemeTextview {
    fn default() -> ThemeTextview {
        ThemeTextview {
            background: Color::new(0xff, 0xff, 0xff, 0xff),
            foreground: Color::new(0, 0, 0, 0xff),
            cursor_line: Color::new(0xee, 0xee, 0xee, 0xff),
            cursor: Color::new(0xff, 0x88, 0x22, 0xff),
            border_width: 1,
            border_color: Color::new(0, 0, 0, 0xff),
            indent_guide: Color::new(0xee, 0xee, 0xee, 0x88),
            lint_warnings: Some(Color::new(0x88, 0x88, 0x22, 0xff)),
            lint_errors: Some(Color::new(0xff, 0x22, 0x22, 0xff)),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ThemeGutter {
    pub(crate) background: Color,
    pub(crate) foreground: Color,
}

impl Default for ThemeGutter {
    fn default() -> ThemeGutter {
        ThemeGutter {
            background: Color::new(0xff, 0xff, 0xff, 0xff),
            foreground: Color::new(0, 0, 0, 0x80),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ThemePrompt {
    pub(crate) background: Color,
    pub(crate) foreground: Color,
    pub(crate) cursor: Color,
}

impl Default for ThemePrompt {
    fn default() -> ThemePrompt {
        ThemePrompt {
            background: Color::new(0xff, 0xff, 0xff, 0xff),
            foreground: Color::new(0, 0, 0, 0xff),
            cursor: Color::new(0xff, 0x88, 0x22, 0xff),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ThemeCompletion {
    pub(crate) background: Color,
    pub(crate) foreground: Color,
    pub(crate) active_background: Color,
    #[serde(rename(deserialize = "path.directory"))]
    pub(crate) path_directory: Color,
    #[serde(rename(deserialize = "path.file"))]
    pub(crate) path_file: Color,
}

impl Default for ThemeCompletion {
    fn default() -> ThemeCompletion {
        ThemeCompletion {
            background: Color::new(0xee, 0xee, 0xee, 0xff),
            foreground: Color::new(0x22, 0x22, 0x22, 0xff),
            active_background: Color::new(0xff, 0xff, 0xff, 0xff),
            path_file: Color::new(0xff, 0xd5, 0x80, 0xff),
            path_directory: Color::new(0x5c, 0xcf, 0x36, 0xff),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
pub(crate) struct ThemeSyntaxElem {
    pub(crate) foreground: Color,
    #[serde(default)]
    pub(crate) slant: TextSlant,
    #[serde(default)]
    pub(crate) weight: TextWeight,
}

#[derive(Deserialize)]
pub(crate) struct Theme {
    pub(crate) textview: ThemeTextview,
    pub(crate) gutter: ThemeGutter,
    pub(crate) completion: ThemeCompletion,
    pub(crate) prompt: ThemePrompt,
    pub(crate) syntax: FnvHashMap<String, ThemeSyntaxElem>,
}

impl Default for Theme {
    fn default() -> Theme {
        Theme {
            textview: ThemeTextview::default(),
            gutter: ThemeGutter::default(),
            completion: ThemeCompletion::default(),
            prompt: ThemePrompt::default(),
            syntax: FnvHashMap::default(),
        }
    }
}

#[serde(transparent)]
#[derive(Deserialize)]
pub(crate) struct ThemeSet(pub(crate) FnvHashMap<String, Rc<Theme>>);

impl ThemeSet {
    pub(crate) fn load() -> ThemeSet {
        let mut ret_theme_set = ThemeSet::default();
        if let Some(proj_dirs) = ProjectDirs::from("", "sbarua", "bed") {
            // Try loading config
            let theme_dir_path = proj_dirs.config_dir().join("themes");
            for e in WalkDir::new(&theme_dir_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = e.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(theme_set) = read_to_string(e.path()).ok().and_then(|data| {
                        match serde_json::from_str::<ThemeSet>(&data) {
                            Ok(t) => Some(t),
                            Err(err) => {
                                println!("error loading theme: {:?}: {}", e.path(), err);
                                None
                            }
                        }
                    }) {
                        ret_theme_set.0.extend(theme_set.0);
                    }
                }
            }
        }
        ret_theme_set
    }
}

impl Default for ThemeSet {
    fn default() -> ThemeSet {
        let mut themes = FnvHashMap::default();
        themes.insert("default".to_owned(), Rc::new(Theme::default()));
        ThemeSet(themes)
    }
}
