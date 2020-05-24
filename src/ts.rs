// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use fnv::FnvHashMap;
use tree_sitter::{Language, Parser};

extern "C" {
    fn tree_sitter_rust() -> Language;
}

pub(crate) struct TsCore {
    languages: Vec<Language>,
    exts: FnvHashMap<String, usize>,
}

impl TsCore {
    pub(crate) fn new() -> TsCore {
        let languages = vec![unsafe { tree_sitter_rust() }];
        let mut exts = FnvHashMap::default();
        exts.insert("rs".to_owned(), 0);
        TsCore {
            languages: languages,
            exts: exts,
        }
    }

    pub(crate) fn parser_from_extension(&self, ext: &str) -> Option<Parser> {
        self.exts.get(ext).map(|i| {
            let mut parser = Parser::new();
            parser
                .set_language(self.languages[*i])
                .expect("failed to set parser language");
            parser
        })
    }
}
