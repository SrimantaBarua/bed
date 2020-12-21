// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::env;

use geom::size2;
use rype::{Direction, Face, Script};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("USAGE: {} /path/to/font text_to_shape", args[0]);
        return;
    }
    if let Err(e) = try_main(args) {
        eprintln!("ERROR: {}", e);
    }
}

fn try_main(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let face = Face::open(&args[1], 0)?;
    let scaled1 = face.scale(12, size2(96, 96));
    let scaled2 = face.scale(24, size2(96, 96));
    let shaped1 = scaled1.shape(&args[2], Script::Default, Direction::LeftToRight);
    let shaped2 = scaled2.shape(&args[2], Script::Latin, Direction::LeftToRight);
    eprintln!("Shaped1: {:#?}", shaped1);
    eprintln!("Shaped2: {:#?}", shaped2);
    Ok(())
}
