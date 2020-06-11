// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::default::Default;

use directories::ProjectDirs;
use fnv::FnvHashMap;
use serde::Deserialize;

use crate::font::{FaceKey, FontCore};
use crate::style::TextSize;

use super::DEFAULT_THEME;

#[cfg(target_os = "linux")]
static DEFAULT_FONT: &'static str = "monospace";
#[cfg(target_os = "windows")]
static DEFAULT_FONT: &'static str = "Consolas";

static DEFAULT_FONT_SIZE: f32 = 8.0;
static DEFAULT_TAB_WIDTH: usize = 8;
static DEFAULT_INDENT_TABS: bool = true;
static DEFAULT_GUTTER_PADDING: u32 = 8;
static DEFAULT_PROMPT_PADDING_HORIZONTAL: u32 = 4;
static DEFAULT_PROMPT_PADDING_VERTICAL: u32 = 2;
static DEFAULT_COMPLETION_PADDING_HORIZONTAL: u32 = 4;
static DEFAULT_COMPLETION_PADDING_VERTICAL: u32 = 2;

fn default_tab_width() -> usize {
    DEFAULT_TAB_WIDTH
}

fn default_indent_tabs() -> bool {
    DEFAULT_INDENT_TABS
}

fn default_completion_annotation() -> String {
    "".to_owned()
}

#[derive(Deserialize)]
pub(crate) struct ConfigLanguage {
    #[serde(
        rename(deserialize = "editor.tab_width"),
        default = "default_tab_width"
    )]
    pub(crate) tab_width: usize,
    #[serde(
        rename(deserialize = "editor.indent_tabs"),
        default = "default_indent_tabs"
    )]
    pub(crate) indent_tabs: bool,
}

#[derive(Default, Deserialize)]
pub(crate) struct ConfigCompletionAnnotation {
    #[serde(rename(deserialize = "path.directory"), default)]
    pub(crate) path_directory: String,
    #[serde(rename(deserialize = "path.file"), default)]
    pub(crate) path_file: String,
}

pub(crate) struct Config {
    pub(crate) theme: String,
    pub(crate) tab_width: usize,
    pub(crate) indent_tabs: bool,
    pub(crate) language: FnvHashMap<String, ConfigLanguage>,
    // Textview
    pub(crate) textview_face: FaceKey,
    pub(crate) textview_font_size: TextSize,
    // Gutter
    pub(crate) gutter_face: FaceKey,
    pub(crate) gutter_font_size: TextSize,
    pub(crate) gutter_padding: u32,
    // Prompt
    pub(crate) prompt_face: FaceKey,
    pub(crate) prompt_font_size: TextSize,
    pub(crate) prompt_padding_vertical: u32,
    pub(crate) prompt_padding_horizontal: u32,
    // Completion
    pub(crate) completion_face: FaceKey,
    pub(crate) completion_font_size: TextSize,
    pub(crate) completion_padding_vertical: u32,
    pub(crate) completion_padding_horizontal: u32,
    pub(crate) completion_annotation: ConfigCompletionAnnotation,
}

impl Config {
    pub(crate) fn load(font_core: &mut FontCore) -> Config {
        if let Some(proj_dirs) = ProjectDirs::from("", "sbarua", "bed") {
            // Try loading config
            let cfg_dir_path = proj_dirs.config_dir();
            std::fs::read_to_string(cfg_dir_path.join("config.json"))
                .ok()
                .and_then(|data| serde_json::from_str::<ConfigInner>(&data).ok())
                .unwrap_or_default()
                .finalize(font_core)
        } else {
            ConfigInner::default().finalize(font_core)
        }
    }
}

#[derive(Default, Deserialize)]
struct ConfigInner {
    theme: Option<String>,
    // Textview
    #[serde(rename(deserialize = "editor.font_family"))]
    textview_font_family: Option<String>,
    #[serde(rename(deserialize = "editor.font_size"))]
    textview_font_size: Option<f32>,
    #[serde(rename(deserialize = "editor.tab_width"))]
    tab_width: Option<usize>,
    #[serde(rename(deserialize = "editor.indent_tabs"))]
    indent_tabs: Option<bool>,
    // Gutter
    #[serde(rename(deserialize = "gutter.font_family"))]
    gutter_font_family: Option<String>,
    #[serde(rename(deserialize = "gutter.font_scale"))]
    gutter_font_scale: Option<f32>,
    #[serde(rename(deserialize = "gutter.padding"))]
    gutter_padding: Option<u32>,
    // Prompt
    #[serde(rename(deserialize = "prompt.font_family"))]
    prompt_font_family: Option<String>,
    #[serde(rename(deserialize = "prompt.font_scale"))]
    prompt_font_scale: Option<f32>,
    #[serde(rename(deserialize = "prompt.padding_vertical"))]
    prompt_padding_vertical: Option<u32>,
    #[serde(rename(deserialize = "prompt.padding_horizontal"))]
    prompt_padding_horizontal: Option<u32>,
    // Completion
    #[serde(rename(deserialize = "completion.font_family"))]
    completion_font_family: Option<String>,
    #[serde(rename(deserialize = "completion.font_scale"))]
    completion_font_scale: Option<f32>,
    #[serde(rename(deserialize = "completion.padding_vertical"))]
    completion_padding_vertical: Option<u32>,
    #[serde(rename(deserialize = "completion.padding_horizontal"))]
    completion_padding_horizontal: Option<u32>,
    #[serde(rename(deserialize = "completion.annotation"), default)]
    completion_annotation: ConfigCompletionAnnotation,
    // Language-specific
    language: FnvHashMap<String, ConfigLanguage>,
}

impl ConfigInner {
    fn finalize(self, font_core: &mut FontCore) -> Config {
        let theme = self.theme.unwrap_or(DEFAULT_THEME.to_owned());
        let tab_width = self.tab_width.unwrap_or(DEFAULT_TAB_WIDTH);
        let indent_tabs = self.indent_tabs.unwrap_or(DEFAULT_INDENT_TABS);
        // Textview
        let textview_face = self
            .textview_font_family
            .and_then(|s| font_core.find(&s))
            .unwrap_or_else(|| font_core.find(DEFAULT_FONT).expect("failed to load font"));
        let textview_font_size =
            TextSize::from_f32(self.textview_font_size.unwrap_or(DEFAULT_FONT_SIZE));
        // Gutter
        let gutter_face = self
            .gutter_font_family
            .and_then(|s| font_core.find(&s))
            .unwrap_or(textview_face);
        let gutter_font_size = textview_font_size.scale(self.gutter_font_scale.unwrap_or(1.0));
        let gutter_padding = self.gutter_padding.unwrap_or(DEFAULT_GUTTER_PADDING);
        // Prompt
        let prompt_face = self
            .prompt_font_family
            .and_then(|s| font_core.find(&s))
            .unwrap_or(textview_face);
        let prompt_font_size = textview_font_size.scale(self.prompt_font_scale.unwrap_or(1.0));
        let prompt_padding_horizontal = self
            .prompt_padding_horizontal
            .unwrap_or(DEFAULT_PROMPT_PADDING_HORIZONTAL);
        let prompt_padding_vertical = self
            .prompt_padding_vertical
            .unwrap_or(DEFAULT_PROMPT_PADDING_VERTICAL);
        // Completion
        let completion_face = self
            .completion_font_family
            .and_then(|s| font_core.find(&s))
            .unwrap_or(textview_face);
        let completion_font_size =
            textview_font_size.scale(self.completion_font_scale.unwrap_or(1.0));
        let completion_padding_horizontal = self
            .completion_padding_horizontal
            .unwrap_or(DEFAULT_COMPLETION_PADDING_HORIZONTAL);
        let completion_padding_vertical = self
            .completion_padding_vertical
            .unwrap_or(DEFAULT_COMPLETION_PADDING_VERTICAL);
        // Return
        Config {
            theme,
            tab_width,
            indent_tabs,
            language: self.language,
            textview_face,
            textview_font_size,
            gutter_face,
            gutter_font_size,
            gutter_padding,
            prompt_face,
            prompt_font_size,
            prompt_padding_vertical,
            prompt_padding_horizontal,
            completion_face,
            completion_font_size,
            completion_padding_vertical,
            completion_padding_horizontal,
            completion_annotation: self.completion_annotation,
        }
    }
}
