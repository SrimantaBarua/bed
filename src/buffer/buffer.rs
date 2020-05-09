// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::File;
use std::io::Result as IOResult;
use std::rc::Rc;

use euclid::Rect;
use fnv::FnvHashMap;
use ropey::Rope;

use crate::common::PixelSize;
use crate::font::FontCore;

use super::view::BufferView;
use super::BufferViewID;

pub(crate) struct Buffer {
    font_core: Rc<RefCell<FontCore>>,
    data: Rope,
    views: FnvHashMap<BufferViewID, BufferView>,
}

impl Buffer {
    pub(crate) fn new_view(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views.insert(id.clone(), BufferView::new(rect));
    }

    pub(crate) fn set_view_rect(&mut self, id: &BufferViewID, rect: Rect<u32, PixelSize>) {
        self.views.get_mut(id).unwrap().set_rect(rect);
    }

    pub(crate) fn draw_view(&self, id: &BufferViewID) {}

    pub(crate) fn remove_view(&mut self, id: &BufferViewID) {
        self.views.remove(id);
    }

    pub(super) fn empty(font_core: Rc<RefCell<FontCore>>) -> Buffer {
        Buffer {
            font_core: font_core,
            data: Rope::new(),
            views: FnvHashMap::default(),
        }
    }

    pub(super) fn from_file(path: &str, font_core: Rc<RefCell<FontCore>>) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| Buffer {
                font_core: font_core,
                data: rope,
                views: FnvHashMap::default(),
            })
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|mut f| Rope::from_reader(&mut f))
            .map(|rope| self.data = rope)
    }
}
