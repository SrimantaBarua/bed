// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::time;

#[macro_use]
extern crate bitflags;

mod geom;
mod window;

// Central editor state
struct Bed {}

fn main() {
    let (event_loop, window) = window::EventLoop::with_window(geom::size2(800, 600), "bed");
    let target_delta = time::Duration::from_nanos(1_000_000_000 / 60);
    event_loop.run(target_delta, |event| {
        println!("Event: {:?}", event);
    });
}
