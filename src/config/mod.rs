// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::default::Default;

use directories::ProjectDirs;
use fnv::FnvHashMap;
use serde::Deserialize;

use crate::font::{FaceKey, FontCore};
use crate::language::Language;
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
static DEFAULT_HOVER_PADDING_HORIZONTAL: u32 = 4;
static DEFAULT_HOVER_PADDING_VERTICAL: u32 = 2;

pub(crate) struct ConfigLanguage {
    pub(crate) tab_width: usize,
    pub(crate) indent_tabs: bool,
    pub(crate) language_server: Option<ConfigLanguageServer>,
}

#[derive(Deserialize)]
pub(crate) struct ConfigLanguageServer {
    pub(crate) executable: String,
    #[serde(default)]
    pub(crate) arguments: Vec<String>,
    #[serde(
        rename(deserialize = "completion.language_server.root_markers"),
        default
    )]
    pub(crate) root_markers: Vec<String>,
}

#[derive(Deserialize)]
struct ConfigLanguageInner {
    #[serde(rename(deserialize = "editor.tab_width"))]
    tab_width: Option<usize>,
    #[serde(rename(deserialize = "editor.indent_tabs"))]
    indent_tabs: Option<bool>,
    #[serde(rename(deserialize = "completion.language_server"))]
    language_server: Option<ConfigLanguageServer>,
}

impl ConfigLanguageInner {
    fn finalize(self, tab_width: usize, indent_tabs: bool) -> ConfigLanguage {
        ConfigLanguage {
            tab_width: self.tab_width.unwrap_or(tab_width),
            indent_tabs: self.indent_tabs.unwrap_or(indent_tabs),
            language_server: self.language_server,
        }
    }
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
    pub(crate) language: FnvHashMap<Language, ConfigLanguage>,
    // Textview
    pub(crate) textview_face: FaceKey,
    pub(crate) textview_font_size: TextSize,
    pub(crate) textview_line_padding: u32,
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
    pub(crate) completion_line_padding: u32,
    pub(crate) completion_annotation: ConfigCompletionAnnotation,
    pub(crate) completion_langserver_root_markers: Vec<String>,
    // Hover
    pub(crate) hover_face: FaceKey,
    pub(crate) hover_font_size: TextSize,
    pub(crate) hover_padding_vertical: u32,
    pub(crate) hover_padding_horizontal: u32,
    pub(crate) hover_line_padding: u32,
}

impl Config {
    pub(crate) fn load(font_core: &mut FontCore) -> Config {
        if let Some(proj_dirs) = ProjectDirs::from("", "sbarua", "bed") {
            // Try loading config
            let cfg_dir_path = proj_dirs.config_dir();
            std::fs::read_to_string(cfg_dir_path.join("config.json"))
                .ok()
                .and_then(|data| match serde_json::from_str::<ConfigInner>(&data) {
                    Ok(c) => Some(c),
                    Err(e) => {
                        error!("could not parse config: {}", e);
                        None
                    }
                })
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
    #[serde(rename(deserialize = "editor.line_padding"), default)]
    textview_line_padding: u32,
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
    #[serde(rename(deserialize = "completion.line_padding"), default)]
    completion_line_padding: u32,
    #[serde(rename(deserialize = "completion.annotation"), default)]
    completion_annotation: ConfigCompletionAnnotation,
    #[serde(
        rename(deserialize = "completion.language_server.root_markers"),
        default
    )]
    completion_langserver_root_markers: Vec<String>,
    // Hover
    #[serde(rename(deserialize = "hover.font_family"))]
    hover_font_family: Option<String>,
    #[serde(rename(deserialize = "hover.font_scale"))]
    hover_font_scale: Option<f32>,
    #[serde(rename(deserialize = "hover.padding_vertical"))]
    hover_padding_vertical: Option<u32>,
    #[serde(rename(deserialize = "hover.padding_horizontal"))]
    hover_padding_horizontal: Option<u32>,
    #[serde(rename(deserialize = "hover.line_padding"), default)]
    hover_line_padding: u32,
    // Language-specific
    language: FnvHashMap<Language, ConfigLanguageInner>,
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
        let gutter_font_size = textview_font_size.scale({
            let scale = self.gutter_font_scale.unwrap_or(1.0);
            if scale >= 1.0 {
                1.0
            } else {
                scale
            }
        });
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
        // Hover
        let hover_face = self
            .hover_font_family
            .and_then(|s| font_core.find(&s))
            .unwrap_or(textview_face);
        let hover_font_size = textview_font_size.scale(self.hover_font_scale.unwrap_or(1.0));
        let hover_padding_horizontal = self
            .hover_padding_horizontal
            .unwrap_or(DEFAULT_HOVER_PADDING_HORIZONTAL);
        let hover_padding_vertical = self
            .hover_padding_vertical
            .unwrap_or(DEFAULT_HOVER_PADDING_HORIZONTAL);
        // Language config
        let mut language = FnvHashMap::default();
        for (k, v) in self.language {
            language.insert(k, v.finalize(tab_width, indent_tabs));
        }
        // Return
        Config {
            theme,
            tab_width,
            indent_tabs,
            language,
            textview_face,
            textview_font_size,
            textview_line_padding: self.textview_line_padding,
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
            completion_line_padding: self.completion_line_padding,
            completion_annotation: self.completion_annotation,
            completion_langserver_root_markers: self.completion_langserver_root_markers,
            hover_face,
            hover_font_size,
            hover_padding_vertical,
            hover_padding_horizontal,
            hover_line_padding: self.hover_line_padding,
        }
    }
}
