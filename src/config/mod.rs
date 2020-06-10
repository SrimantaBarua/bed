// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::default::Default;

use directories::ProjectDirs;
use fnv::FnvHashMap;
use serde::Deserialize;

use super::{DEFAULT_FONT, DEFAULT_THEME};

#[derive(Deserialize)]
pub(crate) struct ConfigFileType {
    pub(crate) tab_width: usize,
    pub(crate) indent_tabs: bool,
}

#[derive(Deserialize)]
pub(crate) struct Config {
    #[serde(default = "default_theme")]
    pub(crate) theme: String,
    #[serde(default = "default_font_family")]
    pub(crate) font_family: String,
    #[serde(default = "default_font_size")]
    pub(crate) font_size: f32,
    #[serde(default = "default_font_family")]
    pub(crate) gutter_font_family: String,
    #[serde(default = "default_gutter_font_scale")]
    pub(crate) gutter_font_scale: f32,
    #[serde(default = "default_gutter_padding")]
    pub(crate) gutter_padding: u32,
    #[serde(default = "default_font_family")]
    pub(crate) prompt_font_family: String,
    #[serde(default = "default_font_size")]
    pub(crate) prompt_font_size: f32,
    #[serde(default = "default_font_family")]
    pub(crate) completion_font_family: String,
    #[serde(default = "default_completion_font_scale")]
    pub(crate) completion_font_scale: f32,
    #[serde(default = "default_completion_padding")]
    pub(crate) completion_padding: u32,
    #[serde(default = "default_tab_width")]
    pub(crate) tab_width: usize,
    #[serde(default = "default_indent_tabs")]
    pub(crate) indent_tabs: bool,
    #[serde(default)]
    pub(crate) filetypes: FnvHashMap<String, ConfigFileType>,
}

impl Config {
    pub(crate) fn load() -> Config {
        if let Some(proj_dirs) = ProjectDirs::from("", "sbarua", "bed") {
            // Try loading config
            let cfg_dir_path = proj_dirs.config_dir();
            std::fs::read_to_string(cfg_dir_path.join("config.json"))
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default()
        } else {
            Config::default()
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            theme: default_theme(),
            font_family: default_font_family(),
            font_size: default_font_size(),
            gutter_font_family: default_font_family(),
            gutter_font_scale: default_gutter_font_scale(),
            gutter_padding: default_gutter_padding(),
            prompt_font_family: default_font_family(),
            prompt_font_size: default_font_size(),
            completion_font_family: default_font_family(),
            completion_font_scale: default_completion_font_scale(),
            completion_padding: default_completion_padding(),
            tab_width: default_tab_width(),
            indent_tabs: default_indent_tabs(),
            filetypes: FnvHashMap::default(),
        }
    }
}

fn default_theme() -> String {
    DEFAULT_THEME.to_owned()
}

fn default_font_family() -> String {
    DEFAULT_FONT.to_owned()
}

fn default_font_size() -> f32 {
    8.0
}

fn default_gutter_font_scale() -> f32 {
    1.0
}

fn default_gutter_padding() -> u32 {
    8
}

fn default_completion_font_scale() -> f32 {
    1.0
}

fn default_completion_padding() -> u32 {
    4
}

fn default_tab_width() -> usize {
    8
}

fn default_indent_tabs() -> bool {
    true
}
