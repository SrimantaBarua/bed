// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

mod geom;
mod window;

// Central editor state
struct Bed {}

fn main() {
    let mut event_loop = window::EventLoop::new();
    let window = event_loop.new_window(geom::size2(800, 600));
    event_loop.run(|| {});
}
