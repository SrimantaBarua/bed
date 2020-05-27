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
    pub(crate) cursor: Color,
}

impl Default for ThemeTextview {
    fn default() -> ThemeTextview {
        ThemeTextview {
            background: Color::new(0xff, 0xff, 0xff, 0xff),
            foreground: Color::new(0, 0, 0, 0xff),
            cursor: Color::new(0xff, 0x88, 0x22, 0xff),
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
    pub(crate) syntax: FnvHashMap<String, ThemeSyntaxElem>,
}

impl Default for Theme {
    fn default() -> Theme {
        Theme {
            textview: ThemeTextview::default(),
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
