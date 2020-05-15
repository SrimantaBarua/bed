// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::io::Result as IOResult;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use fnv::FnvHashMap;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use super::buffer::Buffer;
use super::hlpool::HlPool;
use super::{BufferID, BufferViewID};

pub(crate) struct BufferMgr {
    buffers: FnvHashMap<String, Weak<RefCell<Buffer>>>,
    syntax_set: Arc<SyntaxSet>,
    theme_set: Arc<ThemeSet>,
    cur_theme: String,
    next_view_id: usize,
    next_buf_id: usize,
    hlpool: Rc<RefCell<HlPool>>,
}

// TODO: Periodically clear out Weak buffers with a strong count of 0

impl BufferMgr {
    pub(crate) fn new(
        syntax_set: Arc<SyntaxSet>,
        theme_set: Arc<ThemeSet>,
        cur_theme: &str,
    ) -> BufferMgr {
        let hlpool = Rc::new(RefCell::new(HlPool::new(
            Arc::clone(&syntax_set),
            Arc::clone(&theme_set),
            cur_theme,
            4,
        )));
        BufferMgr {
            buffers: FnvHashMap::default(),
            next_view_id: 0,
            next_buf_id: 0,
            syntax_set: syntax_set,
            theme_set: theme_set,
            cur_theme: cur_theme.to_owned(),
            hlpool: hlpool,
        }
    }

    pub(crate) fn empty(&mut self) -> Rc<RefCell<Buffer>> {
        let ret = Rc::new(RefCell::new(Buffer::empty(
            BufferID(self.next_buf_id),
            Arc::clone(&self.syntax_set),
            Arc::clone(&self.theme_set),
            &self.cur_theme,
            self.hlpool.clone(),
        )));
        self.next_buf_id += 1;
        ret
    }

    pub(crate) fn from_file(&mut self, path: &str) -> IOResult<Rc<RefCell<Buffer>>> {
        self.buffers
            .get_mut(path)
            .and_then(|weak_ref| weak_ref.upgrade())
            .map(|buffer| {
                (&mut *buffer.borrow_mut())
                    .reload_from_file(path)
                    .map(|_| buffer.clone())
            })
            .unwrap_or_else(|| {
                Buffer::from_file(
                    BufferID(self.next_buf_id),
                    path,
                    Arc::clone(&self.syntax_set),
                    Arc::clone(&self.theme_set),
                    &self.cur_theme,
                    self.hlpool.clone(),
                )
                .map(|buffer| {
                    self.next_buf_id += 1;
                    let buffer = Rc::new(RefCell::new(buffer));
                    self.buffers.insert(path.to_owned(), Rc::downgrade(&buffer));
                    buffer
                })
            })
    }

    pub(crate) fn next_view_id(&mut self) -> BufferViewID {
        let ret = BufferViewID(self.next_view_id);
        self.next_view_id += 1;
        ret
    }
}
