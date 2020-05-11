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
