// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::rc::Rc;

use fnv::FnvHashMap;
use tree_sitter::{Language as TSLanguage, Parser, Query};

use crate::language::Language;

#[link(name = "tslangs")]
extern "C" {
    fn tree_sitter_c() -> TSLanguage;
    fn tree_sitter_cpp() -> TSLanguage;
    fn tree_sitter_css() -> TSLanguage;
    fn tree_sitter_html() -> TSLanguage;
    fn tree_sitter_javascript() -> TSLanguage;
    fn tree_sitter_python() -> TSLanguage;
    fn tree_sitter_rust() -> TSLanguage;
}

static C_HIGHLIGHTS: &str = include_str!("../res/tree-sitter/c/highlights.scm");
static CPP_HIGHLIGHTS: &str = include_str!("../res/tree-sitter/cpp/highlights.scm");
static CSS_HIGHLIGHTS: &str = include_str!("../res/tree-sitter/css/highlights.scm");
static HTML_HIGHLIGHTS: &str = include_str!("../res/tree-sitter/html/highlights.scm");
static JS_HIGHLIGHTS: &str = include_str!("../res/tree-sitter/javascript/highlights.scm");
static PYTHON_HIGHLIGHTS: &str = include_str!("../res/tree-sitter/python/highlights.scm");
static RUST_HIGHLIGHTS: &str = include_str!("../res/tree-sitter/rust/highlights.scm");

pub(crate) struct TsCore {
    languages: Vec<TSLanguage>,
    hl_queries: Vec<Rc<Query>>,
    exts: FnvHashMap<String, (Language, usize)>,
}

impl TsCore {
    pub(crate) fn new() -> TsCore {
        let languages = vec![
            unsafe { tree_sitter_c() },
            unsafe { tree_sitter_cpp() },
            unsafe { tree_sitter_css() },
            unsafe { tree_sitter_html() },
            unsafe { tree_sitter_javascript() },
            unsafe { tree_sitter_python() },
            unsafe { tree_sitter_rust() },
        ];
        let hl_queries = vec![
            Rc::new(
                Query::new(languages[0], C_HIGHLIGHTS)
                    .expect("failed to load highlight queries for C"),
            ),
            Rc::new(
                Query::new(languages[1], &(CPP_HIGHLIGHTS.to_owned() + C_HIGHLIGHTS))
                    .expect("failed to load highlight queries for C++"),
            ),
            Rc::new(
                Query::new(languages[2], CSS_HIGHLIGHTS)
                    .expect("failed to load highlight queries for CSS"),
            ),
            Rc::new(
                Query::new(languages[3], HTML_HIGHLIGHTS)
                    .expect("failed to load highlight queries for HTML"),
            ),
            Rc::new(
                Query::new(languages[4], JS_HIGHLIGHTS)
                    .expect("failed to load highlight queries for JavaScript"),
            ),
            Rc::new(
                Query::new(languages[5], PYTHON_HIGHLIGHTS)
                    .expect("failed to load highlight queries for Python"),
            ),
            Rc::new(
                Query::new(languages[6], RUST_HIGHLIGHTS)
                    .expect("failed to load highlight queries for Rust"),
            ),
        ];
        let mut exts = FnvHashMap::default();
        exts.insert("c".to_owned(), (Language::C, 0));
        exts.insert("h".to_owned(), (Language::C, 0));
        exts.insert("cpp".to_owned(), (Language::Cpp, 1));
        exts.insert("hpp".to_owned(), (Language::Cpp, 1));
        exts.insert("css".to_owned(), (Language::CSS, 2));
        exts.insert("html".to_owned(), (Language::CSS, 3));
        exts.insert("js".to_owned(), (Language::JavaScript, 4));
        exts.insert("py".to_owned(), (Language::Python, 5));
        exts.insert("rs".to_owned(), (Language::Rust, 6));
        TsCore {
            languages,
            hl_queries,
            exts,
        }
    }

    pub(crate) fn parser_from_extension(&self, ext: &str) -> Option<(Language, Parser, Rc<Query>)> {
        self.exts.get(ext).map(|(ft, i)| {
            let mut parser = Parser::new();
            parser
                .set_language(self.languages[*i])
                .expect("failed to set parser language");
            (ft.to_owned(), parser, self.hl_queries[*i].clone())
        })
    }
}
