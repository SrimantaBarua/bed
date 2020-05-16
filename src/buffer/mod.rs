// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

mod buffer;
mod cursor;
mod hlpool;
mod mgr;
mod view;

#[derive(Eq, Hash, PartialEq)]
pub(crate) struct BufferViewID(usize);

impl BufferViewID {
    fn clone(&self) -> BufferViewID {
        BufferViewID(self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct BufferID(usize);

pub(crate) use buffer::Buffer;
pub(crate) use mgr::BufferMgr;
pub(crate) use view::BufferViewCreateParams;
