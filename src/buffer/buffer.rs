// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::File;
use std::io::Result as IOResult;
use std::rc::Rc;

use euclid::Rect;
use fnv::FnvHashMap;
use ropey::Rope;

use crate::common::PixelSize;

use super::view::View;
use super::{BufferBedHandle, BufferViewId};

#[derive(Clone)]
pub(crate) struct BufferHandle(Rc<RefCell<Buffer>>);

impl PartialEq for BufferHandle {
    fn eq(&self, other: &BufferHandle) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for BufferHandle {}

impl BufferHandle {
    pub(crate) fn new_view(&mut self, view_id: &BufferViewId, rect: Rect<f32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        inner.new_view(view_id, rect)
    }

    pub(crate) fn set_view_rect(&mut self, view_id: &BufferViewId, rect: Rect<f32, PixelSize>) {
        let inner = &mut *self.0.borrow_mut();
        inner.set_view_rect(view_id, rect)
    }

    pub(crate) fn draw_view(&mut self, view_id: &BufferViewId) {
        let inner = &mut *self.0.borrow_mut();
        inner.draw_view(view_id)
    }

    pub(super) fn create_empty(bed_handle: BufferBedHandle) -> BufferHandle {
        BufferHandle(Rc::new(RefCell::new(Buffer::empty(bed_handle))))
    }

    pub(super) fn create_from_file(
        path: &str,
        bed_handle: BufferBedHandle,
    ) -> IOResult<BufferHandle> {
        Buffer::from_file(path, bed_handle).map(|buf| BufferHandle(Rc::new(RefCell::new(buf))))
    }

    pub(super) fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        let inner = &mut *self.0.borrow_mut();
        inner.reload_from_file(path)
    }
}

struct Buffer {
    views: FnvHashMap<BufferViewId, View>,
    bed_handle: BufferBedHandle,
    rope: Rope,
}

impl Buffer {
    // -------- View manipulation --------
    fn new_view(&mut self, view_id: &BufferViewId, rect: Rect<f32, PixelSize>) {
        let view = View::new(self.bed_handle.clone(), rect);
        self.views.insert(view_id.clone(), view);
    }

    fn set_view_rect(&mut self, view_id: &BufferViewId, rect: Rect<f32, PixelSize>) {
        let view = self.views.get_mut(view_id).unwrap();
        view.set_rect(rect);
    }

    fn draw_view(&mut self, view_id: &BufferViewId) {
        let view = self.views.get_mut(view_id).unwrap();
        view.draw(&self.rope);
    }

    // -------- Creation / reading from file --------
    fn empty(bed_handle: BufferBedHandle) -> Buffer {
        Buffer {
            views: FnvHashMap::default(),
            rope: Rope::new(),
            bed_handle,
        }
    }

    fn from_file(path: &str, bed_handle: BufferBedHandle) -> IOResult<Buffer> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| Buffer {
                rope,
                bed_handle,
                views: FnvHashMap::default(),
            })
    }

    fn reload_from_file(&mut self, path: &str) -> IOResult<()> {
        File::open(path)
            .and_then(|f| Rope::from_reader(f))
            .map(|rope| {
                self.rope = rope;
                for view in self.views.values_mut() {
                    view.scroll_to_top();
                }
            })
    }
}
