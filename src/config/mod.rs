// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::default::Default;

use directories::ProjectDirs;
use serde::Deserialize;

use super::{DEFAULT_FONT, DEFAULT_THEME};

static DEFAULT_FONT_SIZE: f32 = 8.0;
static DEFAULT_GUTTER_FONT_SCALE: f32 = 0.8;
static DEFAULT_GUTTER_PADDING: u32 = 8;

pub(crate) struct Config {
    pub(crate) theme: String,
    pub(crate) font_family: String,
    pub(crate) font_size: f32,
    pub(crate) gutter_font_family: String,
    pub(crate) gutter_font_scale: f32,
    pub(crate) gutter_padding: u32,
}

impl Config {
    pub(crate) fn load() -> Config {
        if let Some(proj_dirs) = ProjectDirs::from("", "sbarua", "bed") {
            // Try loading config
            let cfg_dir_path = proj_dirs.config_dir();
            std::fs::read_to_string(cfg_dir_path.join("config.json"))
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .map(|ci: ConfigInner| ci.build())
                .unwrap_or_default()
        } else {
            Config::default()
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            theme: DEFAULT_THEME.to_owned(),
            font_family: DEFAULT_FONT.to_owned(),
            font_size: DEFAULT_FONT_SIZE,
            gutter_font_family: DEFAULT_FONT.to_owned(),
            gutter_font_scale: DEFAULT_GUTTER_FONT_SCALE,
            gutter_padding: DEFAULT_GUTTER_PADDING,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigInner {
    theme: Option<String>,
    font_family: Option<String>,
    font_size: Option<f32>,
    gutter_font_family: Option<String>,
    gutter_font_scale: Option<f32>,
    gutter_padding: Option<u32>,
}

impl ConfigInner {
    fn build(self) -> Config {
        let mut gutter_scale = self.gutter_font_scale.unwrap_or(DEFAULT_GUTTER_FONT_SCALE);
        if gutter_scale > 1.0 {
            gutter_scale = 1.0;
        }
        Config {
            theme: self.theme.unwrap_or(DEFAULT_THEME.to_owned()),
            font_family: self.font_family.unwrap_or(DEFAULT_FONT.to_owned()),
            font_size: self.font_size.unwrap_or(DEFAULT_FONT_SIZE.to_owned()),
            gutter_font_family: self.gutter_font_family.unwrap_or(DEFAULT_FONT.to_owned()),
            gutter_font_scale: gutter_scale,
            gutter_padding: self.gutter_padding.unwrap_or(DEFAULT_GUTTER_PADDING),
        }
    }
}
