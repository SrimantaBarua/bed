// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use language_client::LanguageKey;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Language {
    C,
    Cpp,
    CSS,
    HTML,
    JavaScript,
    Python,
    Rust,
}

impl LanguageKey for Language {}
