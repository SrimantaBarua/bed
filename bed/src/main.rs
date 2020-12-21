// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::time;

#[macro_use]
extern crate bitflags;

mod window;

// Central editor state
struct Bed {}

fn main() {
    let mut wm = window::WindowManager::connect();
    let window = wm.new_window(geom::size2(800, 600), "bed");
    let target_delta = time::Duration::from_nanos(1_000_000_000 / 60);
    wm.run(target_delta, |event| match event {
        window::Event::Refresh(_) => {}
        _ => println!("Event: {:?}", event),
    });
}
