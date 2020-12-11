// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::env;

use rype::Face;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("USAGE: {} /path/to/font", args[0]);
        return;
    }
    if let Err(e) = try_main(args) {
        eprintln!("ERROR: {}", e);
    }
}

fn try_main(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let face = Face::open(&args[1], 0)?;
    eprintln!("Face: {:#?}", face);
    Ok(())
}
