// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

#[cfg(debug_assertions)]
fn main() {
    println!("cargo:rustc-link-search=target/build_debug");
    link_cppstdlib();
}

#[cfg(not(debug_assertions))]
fn main() {
    println!("cargo:rustc-link-search=target/build_release");
    link_cppstdlib();
}

#[cfg(target_os = "linux")]
fn link_cppstdlib() {
    println!("cargo:rustc-link-lib=dylib=stdc++");
}

#[cfg(not(target_os = "linux"))]
fn link_cppstdlib() {
    unimplemented!()
}
