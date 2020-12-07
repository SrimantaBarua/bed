// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

#[macro_use]
extern crate bitflags;

mod error;
mod face;
mod font_collection;
mod head;
mod hhea;
mod hmtx;
mod maxp;
mod rcbuffer;
mod types;

pub use error::*;
pub use face::Face;
pub use font_collection::FontCollection;
