cmake_minimum_required(VERSION 3.1)
project(tslangs VERSION 0.1 LANGUAGES C CXX)

add_library(tslangs STATIC)

target_sources(tslangs PRIVATE c/src/parser.c)
target_include_directories(tslangs PUBLIC c/src)

target_sources(tslangs PRIVATE cpp/src/parser.c cpp/src/scanner.cc)
target_include_directories(tslangs PUBLIC cpp/src)

target_sources(tslangs PRIVATE css/src/parser.c css/src/scanner.c)
target_include_directories(tslangs PUBLIC css/src)

target_sources(tslangs PRIVATE html/src/parser.c html/src/scanner.cc)
target_include_directories(tslangs PUBLIC html/src)

target_sources(tslangs PRIVATE javascript/src/parser.c javascript/src/scanner.c)
target_include_directories(tslangs PUBLIC javascript/src)

target_sources(tslangs PRIVATE python/src/parser.c python/src/scanner.cc)
target_include_directories(tslangs PUBLIC python/src)

target_sources(tslangs PRIVATE rust/src/parser.c rust/src/scanner.c)
target_include_directories(tslangs PUBLIC rust/src)
