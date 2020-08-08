// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::time;

mod geom;
mod window;

// Central editor state
struct Bed {}

fn main() {
    let mut event_loop = window::EventLoop::new();
    let window = event_loop.new_window(geom::size2(800, 600));
    let target_delta = time::Duration::from_nanos(1_000_000_000 / 60);
    event_loop.run(target_delta, |event| {
        println!("Event: {:?}", event);
    });
}
