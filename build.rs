// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::path::PathBuf;

fn main() {
    // ---- Tree-sitter --------
    // tree-sitter-rust
    let dir = PathBuf::from("res/tree-sitter/rust/src");
    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .compile("tree-sitter-rust");
}
