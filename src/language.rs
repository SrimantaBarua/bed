// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

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

impl Language {
    pub(crate) fn to_str(&self) -> &'static str {
        match self {
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::CSS => "css",
            Language::HTML => "html",
            Language::JavaScript => "javascript",
            Language::Python => "python",
            Language::Rust => "rust",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}
