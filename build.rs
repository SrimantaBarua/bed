// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

extern crate cc;

fn build_treesitter_bash() {
    cc::Build::new()
        .cpp(true)
        .file("res/tree-sitter/bash/src/scanner.cc")
        .include("res/tree-sitter/bash/src")
        .warnings(false)
        .compile("ts_bash_scanner");
    cc::Build::new()
        .file("res/tree-sitter/bash/src/parser.c")
        .include("res/tree-sitter/bash/src")
        .warnings(false)
        .compile("ts_bash");
}

fn build_treesitter_c() {
    cc::Build::new()
        .file("res/tree-sitter/c/src/parser.c")
        .include("res/tree-sitter/c/src")
        .warnings(false)
        .compile("ts_c");
}

fn build_treesitter_cpp() {
    cc::Build::new()
        .cpp(true)
        .file("res/tree-sitter/cpp/src/scanner.cc")
        .include("res/tree-sitter/cpp/src")
        .warnings(false)
        .compile("ts_cpp_scanner");
    cc::Build::new()
        .file("res/tree-sitter/cpp/src/parser.c")
        .include("res/tree-sitter/cpp/src")
        .warnings(false)
        .compile("ts_cpp");
}

fn build_treesitter_css() {
    cc::Build::new()
        .file("res/tree-sitter/css/src/parser.c")
        .file("res/tree-sitter/css/src/scanner.c")
        .include("res/tree-sitter/css/src")
        .warnings(false)
        .compile("ts_css");
}

fn build_treesitter_html() {
    cc::Build::new()
        .cpp(true)
        .file("res/tree-sitter/html/src/scanner.cc")
        .include("res/tree-sitter/html/src")
        .warnings(false)
        .compile("ts_html_scanner");
    cc::Build::new()
        .file("res/tree-sitter/html/src/parser.c")
        .include("res/tree-sitter/html/src")
        .warnings(false)
        .compile("ts_html");
}

fn build_treesitter_javascript() {
    cc::Build::new()
        .file("res/tree-sitter/javascript/src/parser.c")
        .file("res/tree-sitter/javascript/src/scanner.c")
        .include("res/tree-sitter/javascript/src")
        .warnings(false)
        .compile("ts_javascript");
}

fn build_treesitter_lua() {
    cc::Build::new()
        .cpp(true)
        .file("res/tree-sitter/lua/src/scanner.cc")
        .include("res/tree-sitter/lua/src")
        .warnings(false)
        .compile("ts_lua_scanner");
    cc::Build::new()
        .file("res/tree-sitter/lua/src/parser.c")
        .include("res/tree-sitter/lua/src")
        .warnings(false)
        .compile("ts_lua");
}

fn build_treesitter_markdown() {
    cc::Build::new()
        .cpp(true)
        .file("res/tree-sitter/markdown/src/scanner.cc")
        .include("res/tree-sitter/markdown/src")
        .warnings(false)
        .compile("ts_markdown_scanner");
    cc::Build::new()
        .file("res/tree-sitter/markdown/src/parser.c")
        .include("res/tree-sitter/markdown/src")
        .warnings(false)
        .compile("ts_markdown");
}

fn build_treesitter_python() {
    cc::Build::new()
        .cpp(true)
        .file("res/tree-sitter/python/src/scanner.cc")
        .include("res/tree-sitter/python/src")
        .warnings(false)
        .compile("ts_python_scanner");
    cc::Build::new()
        .file("res/tree-sitter/python/src/parser.c")
        .include("res/tree-sitter/python/src")
        .warnings(false)
        .compile("ts_python");
}

fn build_treesitter_rust() {
    cc::Build::new()
        .file("res/tree-sitter/rust/src/parser.c")
        .file("res/tree-sitter/rust/src/scanner.c")
        .include("res/tree-sitter/rust/src")
        .warnings(false)
        .compile("ts_rust");
}

fn build_treesitter_toml() {
    cc::Build::new()
        .file("res/tree-sitter/toml/src/parser.c")
        .file("res/tree-sitter/toml/src/scanner.c")
        .include("res/tree-sitter/toml/src")
        .warnings(false)
        .compile("ts_toml");
}

fn build_treesitter() {
    build_treesitter_bash();
    build_treesitter_c();
    build_treesitter_cpp();
    build_treesitter_css();
    build_treesitter_html();
    build_treesitter_javascript();
    build_treesitter_lua();
    build_treesitter_markdown();
    build_treesitter_python();
    build_treesitter_rust();
    build_treesitter_toml();
}

fn main() {
    build_treesitter()
}
