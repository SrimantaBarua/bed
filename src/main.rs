// Second attempt at writing a text editor in Rust. This incorporates all my learning
// from the first attempt

// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

#[macro_use]
extern crate clap;

use euclid::size2;

use ::bed::Bed;

fn main() {
    let args = parse_args();
    let size = size2(800, 600);
    Bed::run(args, size);
}

fn parse_args() -> clap::ArgMatches<'static> {
    use clap::{App, Arg};
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("FILE")
                .help("file to open")
                .required(false)
                .index(1),
        )
        .get_matches()
}
