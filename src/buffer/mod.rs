// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;

use crate::config::Config;
use crate::style::TextSize;
use crate::text::FontCollectionHandle;
use crate::theme::{Theme, ThemeSet};

mod buffer;
mod mgr;
mod rope_stuff;
mod view;

pub(crate) use buffer::{Buffer, BufferHandle};
pub(crate) use mgr::BufferMgr;
pub(crate) use view::CursorStyle;

// Handle to BufferView
#[derive(Eq, PartialEq, Hash)]
pub(crate) struct BufferViewId(usize);

impl BufferViewId {
    fn clone(&self) -> BufferViewId {
        BufferViewId(self.0)
    }
}

// Handle to editor state for buffer module
#[derive(Clone)]
pub(crate) struct BufferBedHandle(Rc<RefCell<BufferBedState>>);

impl BufferBedHandle {
    pub(crate) fn new(config: Rc<Config>, theme_set: Rc<ThemeSet>) -> BufferBedHandle {
        BufferBedHandle(Rc::new(RefCell::new(BufferBedState {
            needs_redraw: false,
            text_font: config.textview_font.clone(),
            text_size: config.textview_font_size.clone(),
            theme: config.theme.clone(),
            theme_set,
        })))
    }

    pub(crate) fn set_text_font(&mut self, font: FontCollectionHandle) {
        self.0.borrow_mut().text_font = font;
    }

    pub(crate) fn scale_text(&mut self, scale: f64) {
        let mut inner = self.0.borrow_mut();
        inner.text_size = inner.text_size.scale(scale);
    }

    pub(crate) fn collect_redraw_state(&mut self) -> bool {
        let mut inner = self.0.borrow_mut();
        let ret = inner.needs_redraw;
        inner.needs_redraw = false;
        ret
    }

    fn text_font(&self) -> FontCollectionHandle {
        self.0.borrow().text_font.clone()
    }

    fn text_size(&self) -> TextSize {
        self.0.borrow().text_size
    }

    fn theme(&self) -> ThemeGuard {
        ThemeGuard(self.0.borrow())
    }

    fn request_redraw(&mut self) {
        self.0.borrow_mut().needs_redraw = true;
    }
}

struct BufferBedState {
    needs_redraw: bool,
    text_font: FontCollectionHandle,
    text_size: TextSize,
    theme: String,
    theme_set: Rc<ThemeSet>,
}

struct ThemeGuard<'a>(Ref<'a, BufferBedState>);

impl<'a> Deref for ThemeGuard<'a> {
    type Target = Theme;

    fn deref(&self) -> &Theme {
        self.0.theme_set.get(&self.0.theme)
    }
}
