// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fs::read_to_string;
use std::rc::Rc;

use fnv::FnvHashMap;
use tree_sitter::{Language as TSLanguage, Parser, Query};

use crate::language::Language;

extern "C" {
    fn tree_sitter_bash() -> TSLanguage;
    fn tree_sitter_c() -> TSLanguage;
    fn tree_sitter_cpp() -> TSLanguage;
    fn tree_sitter_css() -> TSLanguage;
    fn tree_sitter_html() -> TSLanguage;
    fn tree_sitter_javascript() -> TSLanguage;
    fn tree_sitter_lua() -> TSLanguage;
    fn tree_sitter_markdown() -> TSLanguage;
    fn tree_sitter_python() -> TSLanguage;
    fn tree_sitter_rust() -> TSLanguage;
    fn tree_sitter_toml() -> TSLanguage;
}

const LANGUAGES: [&'static str; 11] = [
    "bash",
    "c",
    "cpp",
    "css",
    "html",
    "javascript",
    "lua",
    "markdown",
    "python",
    "rust",
    "toml",
];

const QUERY_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/res/tree-sitter/queries");

fn highlight_paths() -> impl Iterator<Item = String> {
    LANGUAGES
        .iter()
        .map(|lang| QUERY_DIR.to_owned() + "/" + lang + "/highlight.scm")
}

fn fold_paths() -> impl Iterator<Item = String> {
    LANGUAGES
        .iter()
        .map(|lang| QUERY_DIR.to_owned() + "/" + lang + "/fold.scm")
}

fn indent_paths() -> impl Iterator<Item = String> {
    LANGUAGES
        .iter()
        .map(|lang| QUERY_DIR.to_owned() + "/" + lang + "/indent.scm")
}

pub(crate) struct TsLang {
    pub(crate) parser: Parser,
    pub(crate) hl_query: Rc<Query>,
    pub(crate) fold_query: Rc<Query>,
    pub(crate) indent_query: Rc<Query>,
}

pub(crate) struct TsCore {
    languages: Vec<TSLanguage>,
    highlights: Vec<Rc<Query>>,
    folds: Vec<Rc<Query>>,
    indents: Vec<Rc<Query>>,
    exts: FnvHashMap<&'static str, (Language, usize)>,
}

impl TsCore {
    pub(crate) fn new() -> TsCore {
        let languages = unsafe {
            vec![
                tree_sitter_bash(),
                tree_sitter_c(),
                tree_sitter_cpp(),
                tree_sitter_css(),
                tree_sitter_html(),
                tree_sitter_javascript(),
                tree_sitter_lua(),
                tree_sitter_markdown(),
                tree_sitter_python(),
                tree_sitter_rust(),
                tree_sitter_toml(),
            ]
        };
        let query_err = |e, i, s| {
            eprintln!("failed to load {}: {}: {:?}", s, i, e);
            Query::new(languages[i], "").unwrap()
        };
        let highlights = highlight_paths()
            .enumerate()
            .map(|(i, s)| {
                let b = read_to_string(&s).expect("failed to read highlights");
                Rc::new(Query::new(languages[i], &b).unwrap_or_else(|e| query_err(e, i, s)))
            })
            .collect();
        let folds = fold_paths()
            .enumerate()
            .map(|(i, s)| {
                let b = read_to_string(&s).expect("failed to read folds");
                Rc::new(Query::new(languages[i], &b).unwrap_or_else(|e| query_err(e, i, s)))
            })
            .collect();
        let indents = indent_paths()
            .enumerate()
            .map(|(i, s)| {
                let b = read_to_string(&s).expect("failed to read indents");
                Rc::new(Query::new(languages[i], &b).unwrap_or_else(|e| query_err(e, i, s)))
            })
            .collect();
        let mut exts = FnvHashMap::default();
        exts.insert("sh", (Language::Bash, 0));
        exts.insert("c", (Language::C, 1));
        exts.insert("h", (Language::C, 1));
        exts.insert("cpp", (Language::Cpp, 2));
        exts.insert("hpp", (Language::Cpp, 2));
        exts.insert("css", (Language::CSS, 3));
        exts.insert("html", (Language::HTML, 4));
        exts.insert("js", (Language::JavaScript, 5));
        exts.insert("lua", (Language::Lua, 6));
        exts.insert("md", (Language::Markdown, 7));
        exts.insert("py", (Language::Python, 8));
        exts.insert("rs", (Language::Rust, 9));
        exts.insert("toml", (Language::Toml, 10));
        TsCore {
            languages,
            exts,
            highlights,
            folds,
            indents,
        }
    }

    pub(crate) fn parser_from_extension(&self, ext: &str) -> Option<(Language, TsLang)> {
        self.exts.get(ext).map(|(ft, i)| {
            let mut parser = Parser::new();
            parser
                .set_language(self.languages[*i])
                .expect("failed to set parser language");
            let hl_query = self.highlights[*i].clone();
            let fold_query = self.folds[*i].clone();
            let indent_query = self.indents[*i].clone();
            (
                ft.to_owned(),
                TsLang {
                    parser,
                    hl_query,
                    fold_query,
                    indent_query,
                },
            )
        })
    }
}
