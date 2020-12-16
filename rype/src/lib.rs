// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

#[macro_use]
extern crate bitflags;

mod classdef;
mod cmap;
mod coverage;
mod ctx_lookup;
mod error;
mod face;
mod featurelist;
mod gasp;
mod gdef;
mod gpos;
mod gsub;
mod head;
mod hhea;
mod hmtx;
mod kern;
mod lookuplist;
mod maxp;
mod scriptlist;
mod types;

pub use error::*;
pub use face::Face;
