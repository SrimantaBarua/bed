// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use euclid::{size2, Point2D, Rect};

use crate::buffer::{Buffer, BufferViewCreateParams, BufferViewID};
use crate::common::PixelSize;
use crate::painter::Painter;
use crate::theme::Theme;

use super::TextPane;

pub(crate) struct TextTree {
    theme: Rc<Theme>,
    rect: Rect<u32, PixelSize>,
    root: Node,
}

impl TextTree {
    pub(crate) fn new(
        view_params: BufferViewCreateParams,
        buf: Rc<RefCell<Buffer>>,
        view_id: BufferViewID,
        theme: Rc<Theme>,
    ) -> TextTree {
        TextTree {
            rect: view_params.rect,
            root: Node::new_leaf(view_params, buf, view_id),
            theme,
        }
    }

    pub(crate) fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        self.rect = rect;
        self.root.set_rect(rect, self.theme.textview.border_width);
    }

    pub(crate) fn draw(&self, painter: &mut Painter) {
        let _ = painter.widget_ctx(self.rect.cast(), self.theme.textview.border_color, false);
        self.root.draw(painter);
    }

    pub(crate) fn split_v(
        &mut self,
        optbuffer: Option<Rc<RefCell<Buffer>>>,
        view_id: BufferViewID,
    ) {
        self.root.split_v(optbuffer, view_id);
        self.root
            .set_rect(self.rect, self.theme.textview.border_width);
    }

    pub(crate) fn split_h(
        &mut self,
        optbuffer: Option<Rc<RefCell<Buffer>>>,
        view_id: BufferViewID,
    ) {
        self.root.split_h(optbuffer, view_id);
        self.root
            .set_rect(self.rect, self.theme.textview.border_width);
    }

    pub(crate) fn active(&self) -> &TextPane {
        self.root.active()
    }

    pub(crate) fn active_mut(&mut self) -> &mut TextPane {
        self.root.active_mut()
    }

    pub(crate) fn set_active_and_get_from_pos(
        &mut self,
        pos: Point2D<u32, PixelSize>,
    ) -> Option<&mut TextPane> {
        if !self.rect.contains(pos) {
            return None;
        }
        self.root.set_active_and_get_from_pos(pos)
    }

    pub(crate) fn set_hover(&mut self, optpoint: Option<Point2D<u32, PixelSize>>) {
        self.root.set_hover(optpoint);
    }

    pub(crate) fn map<F>(&mut self, f: F) -> bool
    where
        F: FnMut(&mut TextPane) -> bool + Clone,
    {
        self.root.map(f)
    }
}

#[derive(PartialEq, Eq)]
enum Split {
    None,
    Horizontal, // horizontal line between two panes, top and bottom
    Vertical,   // vertical line between two panes, left and right
}

struct Node {
    rect: Rect<u32, PixelSize>,
    split: Split,
    children: Vec<Node>,
    active: usize,
    opt_view: Option<TextPane>,
}

impl Node {
    fn new_leaf(
        view_params: BufferViewCreateParams,
        buf: Rc<RefCell<Buffer>>,
        id: BufferViewID,
    ) -> Node {
        Node {
            rect: view_params.rect,
            split: Split::None,
            children: Vec::new(),
            active: 0,
            opt_view: Some(TextPane::new(view_params, buf, id)),
        }
    }

    fn leaf_with(view: TextPane) -> Node {
        Node {
            rect: view.rect(),
            split: Split::None,
            children: Vec::new(),
            active: 0,
            opt_view: Some(view),
        }
    }

    fn is_leaf(&self) -> bool {
        self.opt_view.is_some()
    }

    fn map<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut TextPane) -> bool + Clone,
    {
        if self.is_leaf() {
            f(self.opt_view.as_mut().unwrap())
        } else {
            let mut ret = false;
            for c in &mut self.children {
                ret |= c.map(f.clone());
            }
            ret
        }
    }

    fn set_hover(&mut self, optpoint: Option<Point2D<u32, PixelSize>>) {
        if self.is_leaf() {
            self.opt_view.as_mut().unwrap().set_hover(optpoint);
        } else {
            match optpoint {
                Some(pos) => {
                    for i in 0..self.children.len() {
                        if self.children[i].rect.contains(pos) {
                            self.children[i].set_hover(Some(pos));
                        } else {
                            self.children[i].set_hover(None);
                        }
                    }
                }
                None => {
                    for i in 0..self.children.len() {
                        self.children[i].set_hover(None);
                    }
                }
            }
        }
    }

    fn draw(&self, painter: &mut Painter) {
        if self.is_leaf() {
            self.opt_view.as_ref().unwrap().draw(painter);
        } else {
            for c in &self.children {
                c.draw(painter);
            }
        }
    }

    fn split_h(&mut self, optbuffer: Option<Rc<RefCell<Buffer>>>, view_id: BufferViewID) {
        if self.is_leaf() {
            let view = self.opt_view.take().unwrap();
            self.active = 0;
            self.children
                .push(Node::leaf_with(view.clone(optbuffer, view_id)));
            self.children.push(Node::leaf_with(view));
            self.split = Split::Horizontal;
        } else if self.split == Split::Horizontal {
            if self.children[self.active].is_leaf() {
                let view = self.children[self.active]
                    .opt_view
                    .as_ref()
                    .unwrap()
                    .clone(optbuffer, view_id);
                self.children.insert(self.active, Node::leaf_with(view));
            } else {
                self.children[self.active].split_h(optbuffer, view_id);
            }
        } else {
            self.children[self.active].split_h(optbuffer, view_id);
        }
    }

    fn split_v(&mut self, optbuffer: Option<Rc<RefCell<Buffer>>>, view_id: BufferViewID) {
        if self.is_leaf() {
            let view = self.opt_view.take().unwrap();
            self.active = 0;
            self.children
                .push(Node::leaf_with(view.clone(optbuffer, view_id)));
            self.children.push(Node::leaf_with(view));
            self.split = Split::Vertical;
        } else if self.split == Split::Vertical {
            if self.children[self.active].is_leaf() {
                let view = self.children[self.active]
                    .opt_view
                    .as_ref()
                    .unwrap()
                    .clone(optbuffer, view_id);
                self.children.insert(self.active, Node::leaf_with(view));
            } else {
                self.children[self.active].split_v(optbuffer, view_id);
            }
        } else {
            self.children[self.active].split_v(optbuffer, view_id);
        }
    }

    fn set_rect(&mut self, rect: Rect<u32, PixelSize>, border_width: u32) {
        self.rect = rect;
        if self.is_leaf() {
            self.opt_view.as_mut().unwrap().set_rect(rect);
        } else {
            let num_c = self.children.len() as u32;
            let mut origin = rect.origin;
            if self.split == Split::Vertical {
                let rem_w = rect.size.width - border_width * (num_c - 1);
                let num_p1 = (rem_w % num_c) as usize;
                let w_per_c = rem_w / num_c;
                let mut size = size2(0, rect.size.height);
                for i in 0..self.children.len() {
                    if i < num_p1 {
                        size.width = w_per_c + 1;
                    } else {
                        size.width = w_per_c;
                    }
                    self.children[i].set_rect(Rect::new(origin, size), border_width);
                    origin.x += size.width + border_width;
                }
            } else {
                let rem_h = rect.size.height - border_width * (num_c - 1);
                let num_p1 = (rem_h % num_c) as usize;
                let h_per_c = rem_h / num_c;
                let mut size = size2(rect.size.width, 0);
                for i in 0..self.children.len() {
                    if i < num_p1 {
                        size.height = h_per_c + 1;
                    } else {
                        size.height = h_per_c;
                    }
                    self.children[i].set_rect(Rect::new(origin, size), border_width);
                    origin.y += size.height + border_width;
                }
            }
        }
    }

    fn active(&self) -> &TextPane {
        if let Some(tv) = &self.opt_view {
            tv
        } else {
            self.children[self.active].active()
        }
    }

    fn active_mut(&mut self) -> &mut TextPane {
        if let Some(tv) = &mut self.opt_view {
            tv
        } else {
            self.children[self.active].active_mut()
        }
    }

    fn set_active_and_get_from_pos(
        &mut self,
        pos: Point2D<u32, PixelSize>,
    ) -> Option<&mut TextPane> {
        if self.is_leaf() {
            self.opt_view.as_mut()
        } else {
            for i in 0..self.children.len() {
                if self.children[i].rect.contains(pos) {
                    self.active = i;
                    return self.children[i].set_active_and_get_from_pos(pos);
                }
            }
            None
        }
    }
}
