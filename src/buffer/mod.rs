// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::{Ref, RefCell};
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
pub(crate) struct BufferBedHandle {
    config: Rc<RefCell<Config>>,
    theme_set: Rc<ThemeSet>,
    needs_redraw: Rc<RefCell<bool>>,
}

impl BufferBedHandle {
    pub(crate) fn new(config: Rc<RefCell<Config>>, theme_set: Rc<ThemeSet>) -> BufferBedHandle {
        BufferBedHandle {
            needs_redraw: Rc::new(RefCell::new(false)),
            config,
            theme_set,
        }
    }

    pub(crate) fn collect_redraw_state(&mut self) -> bool {
        let mut needs_redraw = self.needs_redraw.borrow_mut();
        let ret = *needs_redraw;
        *needs_redraw = false;
        ret
    }

    fn text_line_pad(&self) -> u32 {
        self.config().textview_line_padding
    }

    fn text_font(&self) -> FontCollectionHandle {
        self.config().textview_font.clone()
    }

    fn text_font_size(&self) -> TextSize {
        self.config().textview_font_size
    }

    fn gutter_font(&self) -> FontCollectionHandle {
        self.config().gutter_font.clone()
    }

    fn gutter_font_scale(&self) -> f64 {
        self.config().gutter_font_scale
    }

    fn gutter_padding(&self) -> u32 {
        self.config().gutter_padding
    }

    fn theme(&self) -> &Theme {
        self.theme_set.get(&self.config.borrow().theme)
    }

    fn config(&self) -> Ref<Config> {
        self.config.borrow()
    }

    fn request_redraw(&mut self) {
        let mut needs_redraw = self.needs_redraw.borrow_mut();
        *needs_redraw = true;
    }
}
