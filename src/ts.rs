// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::rc::Rc;

use fnv::FnvHashMap;
use tree_sitter::{Language, Parser, Query};

extern "C" {
    fn tree_sitter_c() -> Language;
    fn tree_sitter_rust() -> Language;
}

static C_HIGHLIGHTS: &str = include_str!("../res/tree-sitter/c/highlights.scm");
static RUST_HIGHLIGHTS: &str = include_str!("../res/tree-sitter/rust/highlights.scm");

pub(crate) struct TsCore {
    languages: Vec<Language>,
    hl_queries: Vec<Rc<Query>>,
    exts: FnvHashMap<String, usize>,
}

impl TsCore {
    pub(crate) fn new() -> TsCore {
        let languages = vec![unsafe { tree_sitter_c() }, unsafe { tree_sitter_rust() }];
        let hl_queries = vec![
            Rc::new(
                Query::new(languages[0], C_HIGHLIGHTS)
                    .expect("failed to load highlight queries for C"),
            ),
            Rc::new(
                Query::new(languages[1], RUST_HIGHLIGHTS)
                    .expect("failed to load highlight queries for Rust"),
            ),
        ];
        let mut exts = FnvHashMap::default();
        exts.insert("c".to_owned(), 0);
        exts.insert("h".to_owned(), 0);
        exts.insert("rs".to_owned(), 1);
        TsCore {
            languages: languages,
            hl_queries: hl_queries,
            exts: exts,
        }
    }

    pub(crate) fn parser_from_extension(&self, ext: &str) -> Option<(Parser, Rc<Query>)> {
        self.exts.get(ext).map(|i| {
            let mut parser = Parser::new();
            parser
                .set_language(self.languages[*i])
                .expect("failed to set parser language");
            (parser, self.hl_queries[*i].clone())
        })
    }
}
