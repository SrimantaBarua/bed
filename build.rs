// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::path::PathBuf;

fn main() {
    // ---- Tree-sitter --------
    // tree-sitter-c
    let cdir = PathBuf::from("res/tree-sitter/c/src");
    cc::Build::new()
        .include(&cdir)
        .file(cdir.join("parser.c"))
        .compile("tree-sitter-c");

    // tree-sitter-rust
    let rsdir = PathBuf::from("res/tree-sitter/rust/src");
    cc::Build::new()
        .include(&rsdir)
        .file(rsdir.join("parser.c"))
        .file(rsdir.join("scanner.c"))
        .compile("tree-sitter-rust");
}
