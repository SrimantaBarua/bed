#!/usr/bin/env bash

if [ $# == 1 ] && [ "$1" == "release" ]; then

    mkdir -p target/build_release
    cd target/build_release
    cmake -DCMAKE_BUILD_TYPE=Release ../../res/tree-sitter
    make -j $(nproc)
    cd ../..
    cargo build --release

else

    mkdir -p target/build_debug
    cd target/build_debug
    cmake -DCMAKE_BUILD_TYPE=Debug ../../res/tree-sitter
    make -j $(nproc)
    cd ../..
    cargo build

fi
