// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

mod rope;
pub(crate) use {
    rope::rope_is_grapheme_boundary, rope::rope_next_grapheme_boundary, rope::rope_trim_newlines,
    rope::RopeGraphemes,
};

// Types for euclid
pub(crate) struct DPI;
pub struct PixelSize;
pub(crate) struct TextureSize;

pub(crate) fn abspath(spath: &str) -> String {
    let path = std::path::Path::new(spath);
    if path.is_absolute() {
        spath.to_owned()
    } else if path.starts_with("~") {
        let mut home_dir = directories::BaseDirs::new()
            .expect("failed to get base directories")
            .home_dir()
            .to_owned();
        home_dir.push(path.strip_prefix("~").expect("failed to stip '~' prefix"));
        home_dir
            .to_str()
            .expect("failed to convert path to string")
            .to_owned()
    } else {
        let mut wdir = std::env::current_dir().expect("failed to get current directory");
        wdir.push(spath);
        wdir.to_str()
            .expect("failed to convert path to string")
            .to_owned()
    }
}
