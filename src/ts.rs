// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

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

static HIGHLIGHTS: [&str; 11] = [
    include_str!("../res/tree-sitter/queries/bash/highlight.scm"),
    include_str!("../res/tree-sitter/queries/c/highlight.scm"),
    concat!(
        include_str!("../res/tree-sitter/queries/c/highlight.scm"),
        include_str!("../res/tree-sitter/queries/cpp/highlight.scm")
    ),
    include_str!("../res/tree-sitter/queries/css/highlight.scm"),
    include_str!("../res/tree-sitter/queries/html/highlight.scm"),
    include_str!("../res/tree-sitter/queries/javascript/highlight.scm"),
    include_str!("../res/tree-sitter/queries/lua/highlight.scm"),
    include_str!("../res/tree-sitter/queries/markdown/highlight.scm"),
    include_str!("../res/tree-sitter/queries/python/highlight.scm"),
    include_str!("../res/tree-sitter/queries/rust/highlight.scm"),
    include_str!("../res/tree-sitter/queries/toml/highlight.scm"),
];

static FOLDS: [&str; 11] = [
    include_str!("../res/tree-sitter/queries/bash/fold.scm"),
    include_str!("../res/tree-sitter/queries/c/fold.scm"),
    concat!(
        include_str!("../res/tree-sitter/queries/c/fold.scm"),
        include_str!("../res/tree-sitter/queries/cpp/fold.scm")
    ),
    include_str!("../res/tree-sitter/queries/css/fold.scm"),
    include_str!("../res/tree-sitter/queries/html/fold.scm"),
    include_str!("../res/tree-sitter/queries/javascript/fold.scm"),
    include_str!("../res/tree-sitter/queries/lua/fold.scm"),
    include_str!("../res/tree-sitter/queries/markdown/fold.scm"),
    include_str!("../res/tree-sitter/queries/python/fold.scm"),
    include_str!("../res/tree-sitter/queries/rust/fold.scm"),
    include_str!("../res/tree-sitter/queries/toml/fold.scm"),
];

static INDENTS: [&str; 11] = [
    include_str!("../res/tree-sitter/queries/bash/indent.scm"),
    include_str!("../res/tree-sitter/queries/c/indent.scm"),
    concat!(
        include_str!("../res/tree-sitter/queries/c/indent.scm"),
        include_str!("../res/tree-sitter/queries/cpp/indent.scm")
    ),
    include_str!("../res/tree-sitter/queries/css/indent.scm"),
    include_str!("../res/tree-sitter/queries/html/indent.scm"),
    include_str!("../res/tree-sitter/queries/javascript/indent.scm"),
    include_str!("../res/tree-sitter/queries/lua/indent.scm"),
    include_str!("../res/tree-sitter/queries/markdown/indent.scm"),
    include_str!("../res/tree-sitter/queries/python/indent.scm"),
    include_str!("../res/tree-sitter/queries/rust/indent.scm"),
    include_str!("../res/tree-sitter/queries/toml/indent.scm"),
];

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
        let highlights = HIGHLIGHTS
            .iter()
            .enumerate()
            .map(|(i, s)| {
                Rc::new(
                    Query::new(languages[i], s).unwrap_or_else(|e| query_err(e, i, "highlight")),
                )
            })
            .collect();
        let folds = FOLDS
            .iter()
            .enumerate()
            .map(|(i, s)| {
                Rc::new(Query::new(languages[i], s).unwrap_or_else(|e| query_err(e, i, "fold")))
            })
            .collect();
        let indents = INDENTS
            .iter()
            .enumerate()
            .map(|(i, s)| {
                Rc::new(Query::new(languages[i], s).unwrap_or_else(|e| query_err(e, i, "indent")))
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
