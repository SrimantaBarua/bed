// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

mod buffer;
mod mgr;

#[derive(Eq, PartialEq)]
pub(crate) struct BufferViewID(usize);

impl BufferViewID {
    fn clone(&self) -> BufferViewID {
        BufferViewID(self.0)
    }
}

pub(crate) use buffer::Buffer;
pub(crate) use mgr::BufferMgr;
