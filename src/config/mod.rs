// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::default::Default;

use directories::ProjectDirs;
use serde::Deserialize;

use super::{DEFAULT_FONT, DEFAULT_FONT_SIZE, DEFAULT_THEME};

pub(crate) struct Config {
    pub(crate) theme: String,
    pub(crate) font_family: String,
    pub(crate) font_size: f32,
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
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigInner {
    theme: Option<String>,
    font_family: Option<String>,
    font_size: Option<f32>,
}

impl ConfigInner {
    fn build(self) -> Config {
        Config {
            theme: self.theme.unwrap_or(DEFAULT_THEME.to_owned()),
            font_family: self.font_family.unwrap_or(DEFAULT_FONT.to_owned()),
            font_size: self.font_size.unwrap_or(DEFAULT_FONT_SIZE.to_owned()),
        }
    }
}
