// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use euclid::{size2, Rect};

use crate::buffer::{Buffer, BufferViewID};
use crate::common::PixelSize;
use crate::painter::Painter;
use crate::style::Color;

use super::TextPane;

pub(crate) struct TextTree {
    border_width: u32,
    root: Node,
}

impl TextTree {
    pub(crate) fn new(
        rect: Rect<u32, PixelSize>,
        buf: Rc<RefCell<Buffer>>,
        view_id: BufferViewID,
    ) -> TextTree {
        TextTree {
            border_width: 1,
            root: Node::new_leaf(rect, buf, view_id),
        }
    }

    pub(crate) fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        self.root.set_rect(rect, self.border_width);
    }

    pub(crate) fn draw(&self, painter: &mut Painter) {
        self.root.draw(painter);
    }

    pub(crate) fn split_v(&mut self, view_id: BufferViewID) {
        self.root.split_v(view_id);
        self.root.set_rect(self.root.rect, self.border_width);
    }

    pub(crate) fn split_h(&mut self, view_id: BufferViewID) {
        self.root.split_h(view_id);
        self.root.set_rect(self.root.rect, self.border_width);
    }

    pub(crate) fn active(&self) -> &TextPane {
        self.root.active()
    }

    pub(crate) fn active_mut(&mut self) -> &mut TextPane {
        self.root.active_mut()
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
    fn new_leaf(rect: Rect<u32, PixelSize>, buf: Rc<RefCell<Buffer>>, id: BufferViewID) -> Node {
        Node {
            rect: rect,
            split: Split::None,
            children: Vec::new(),
            active: 0,
            opt_view: Some(TextPane::new(buf, id)),
        }
    }

    fn leaf_with(rect: Rect<u32, PixelSize>, view: TextPane) -> Node {
        Node {
            rect: rect,
            split: Split::None,
            children: Vec::new(),
            active: 0,
            opt_view: Some(view),
        }
    }

    fn is_leaf(&self) -> bool {
        self.opt_view.is_some()
    }

    fn draw(&self, painter: &mut Painter) {
        if self.is_leaf() {
            painter.rect(self.rect, Color::new(0xff, 0xff, 0xff, 0xff));
        } else {
            for c in &self.children {
                c.draw(painter);
            }
        }
    }

    fn split_h(&mut self, view_id: BufferViewID) {
        if self.is_leaf() {
            let view = self.opt_view.take().unwrap();
            self.active = 0;
            self.children
                .push(Node::leaf_with(self.rect, view.clone(view_id)));
            self.children.push(Node::leaf_with(self.rect, view));
            self.split = Split::Horizontal;
        } else if self.split == Split::Horizontal {
            if self.children[self.active].is_leaf() {
                let view = self.children[self.active]
                    .opt_view
                    .as_ref()
                    .unwrap()
                    .clone(view_id);
                self.children
                    .insert(self.active, Node::leaf_with(self.rect, view));
            } else {
                self.children[self.active].split_h(view_id);
            }
        } else {
            self.children[self.active].split_h(view_id);
        }
    }

    fn split_v(&mut self, view_id: BufferViewID) {
        if self.is_leaf() {
            let view = self.opt_view.take().unwrap();
            self.active = 0;
            self.children
                .push(Node::leaf_with(self.rect, view.clone(view_id)));
            self.children.push(Node::leaf_with(self.rect, view));
            self.split = Split::Vertical;
        } else if self.split == Split::Vertical {
            if self.children[self.active].is_leaf() {
                let view = self.children[self.active]
                    .opt_view
                    .as_ref()
                    .unwrap()
                    .clone(view_id);
                self.children
                    .insert(self.active, Node::leaf_with(self.rect, view));
            } else {
                self.children[self.active].split_v(view_id);
            }
        } else {
            self.children[self.active].split_v(view_id);
        }
    }

    fn set_rect(&mut self, rect: Rect<u32, PixelSize>, border_width: u32) {
        self.rect = rect;
        if !self.is_leaf() {
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
}
