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
    pub(crate) indent_guide: Option<Color>,
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
            indent_guide: Some(Color::new(0xee, 0xee, 0xee, 0x88)),
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

pub(crate) struct ThemeStatus {
    pub(crate) background: Color,
    pub(crate) foreground: Color,
    pub(crate) left_sep: String,
    pub(crate) right_sep: String,
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

#[derive(Deserialize)]
pub(crate) struct ThemeHover {
    pub(crate) background: Color,
    pub(crate) foreground: Color,
}

impl Default for ThemeHover {
    fn default() -> ThemeHover {
        ThemeHover {
            background: Color::new(0xff, 0xff, 0xff, 0xff),
            foreground: Color::new(0, 0, 0, 0x80),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ThemeSyntaxElem {
    pub(crate) foreground: Color,
    pub(crate) slant: TextSlant,
    pub(crate) weight: TextWeight,
    pub(crate) underline: bool,
    pub(crate) scale: f64,
}

pub(crate) struct Theme {
    pub(crate) textview: ThemeTextview,
    pub(crate) gutter: ThemeGutter,
    pub(crate) status: ThemeStatus,
    pub(crate) hover: ThemeHover,
    pub(crate) completion: ThemeCompletion,
    pub(crate) prompt: ThemePrompt,
    pub(crate) syntax: FnvHashMap<String, ThemeSyntaxElem>,
}

pub(crate) struct ThemeSet(pub(crate) FnvHashMap<String, Rc<Theme>>);

impl ThemeSet {
    pub(crate) fn load() -> ThemeSet {
        let mut ret_theme_set = ThemeSetBuilder::default();
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
                        match serde_json::from_str::<ThemeSetBuilder>(&data) {
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
        ret_theme_set.build()
    }

    pub(crate) fn get(&self, theme: &str) -> &Theme {
        self.0.get(theme).unwrap_or_else(|| self.0.get("default").unwrap())
    }
}

#[serde(transparent)]
#[derive(Deserialize)]
struct ThemeSetBuilder(FnvHashMap<String, ThemeBuilder>);

impl Default for ThemeSetBuilder {
    fn default() -> ThemeSetBuilder {
        let mut themes = FnvHashMap::default();
        themes.insert("default".to_owned(), ThemeBuilder::default());
        ThemeSetBuilder(themes)
    }
}

impl ThemeSetBuilder {
    fn build(self) -> ThemeSet {
        ThemeSet(
            self.0
                .into_iter()
                .map(|(k, v)| (k, Rc::new(v.build())))
                .collect(),
        )
    }
}

#[derive(Deserialize)]
struct ThemeBuilder {
    textview: ThemeTextview,
    gutter: ThemeGutter,
    status: ThemeStatusBuilder,
    hover: ThemeHover,
    completion: ThemeCompletion,
    prompt: ThemePrompt,
    syntax: FnvHashMap<String, ThemeSyntaxElemBuilder>,
}

impl Default for ThemeBuilder {
    fn default() -> ThemeBuilder {
        ThemeBuilder {
            textview: ThemeTextview::default(),
            gutter: ThemeGutter::default(),
            status: ThemeStatusBuilder::default(),
            hover: ThemeHover::default(),
            completion: ThemeCompletion::default(),
            prompt: ThemePrompt::default(),
            syntax: FnvHashMap::default(),
        }
    }
}

impl ThemeBuilder {
    fn build(self) -> Theme {
        let mut syntax = FnvHashMap::default();
        for name in self.syntax.keys() {
            self.build_syntax(name, &mut syntax);
        }
        Theme {
            textview: self.textview,
            gutter: self.gutter,
            status: self.status.build(),
            hover: self.hover,
            completion: self.completion,
            prompt: self.prompt,
            syntax,
        }
    }

    fn build_syntax(&self, name: &str, new: &mut FnvHashMap<String, ThemeSyntaxElem>) {
        if new.contains_key(name) {
            return;
        }
        let old_entry = self.syntax.get(name).unwrap();
        let new_entry = match old_entry {
            ThemeSyntaxElemBuilder::Link(next_name) => {
                self.build_syntax(next_name, new);
                new.get(next_name).unwrap().clone()
            }
            ThemeSyntaxElemBuilder::Data {
                foreground,
                slant,
                weight,
                underline,
                scale,
            } => ThemeSyntaxElem {
                foreground: foreground.unwrap_or(self.textview.foreground),
                slant: slant.unwrap_or_default(),
                weight: weight.unwrap_or_default(),
                underline: underline.unwrap_or(false),
                scale: scale.unwrap_or(1.0),
            },
        };
        new.insert(name.to_owned(), new_entry);
    }
}

#[derive(Deserialize)]
struct ThemeStatusBuilder {
    background: Color,
    foreground: Color,
    left_sep: Option<String>,
    right_sep: Option<String>,
}

impl Default for ThemeStatusBuilder {
    fn default() -> ThemeStatusBuilder {
        ThemeStatusBuilder {
            background: Color::new(0xff, 0xff, 0xff, 0xff),
            foreground: Color::new(0, 0, 0, 0x80),
            left_sep: None,
            right_sep: None,
        }
    }
}

impl ThemeStatusBuilder {
    fn build(self) -> ThemeStatus {
        ThemeStatus {
            background: self.background,
            foreground: self.foreground,
            left_sep: self.left_sep.unwrap_or_else(|| "|".to_owned()),
            right_sep: self.right_sep.unwrap_or_else(|| "|".to_owned()),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ThemeSyntaxElemBuilder {
    Link(String),
    Data {
        foreground: Option<Color>,
        slant: Option<TextSlant>,
        weight: Option<TextWeight>,
        underline: Option<bool>,
        scale: Option<f64>,
    },
}
