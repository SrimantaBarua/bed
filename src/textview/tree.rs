// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use euclid::{size2, vec2, Point2D, Rect, Vector2D};
use take_mut::take;

use crate::buffer::{BufferHandle, BufferViewId};
use crate::common::PixelSize;
use crate::config::Config;
use crate::painter::Painter;
use crate::style::Color;
use crate::text::CursorStyle;
use crate::theme::ThemeSet;

use super::view::TextView;

pub(crate) struct TextTree {
    theme_set: Rc<ThemeSet>,
    config: Rc<RefCell<Config>>,
    root: Node,
}

impl TextTree {
    pub(crate) fn new(
        rect: Rect<u32, PixelSize>,
        buffer: BufferHandle,
        view_id: BufferViewId,
        config: Rc<RefCell<Config>>,
        theme_set: Rc<ThemeSet>,
    ) -> TextTree {
        TextTree {
            root: Node::new_leaf(rect, buffer, view_id),
            config,
            theme_set,
        }
    }

    pub(crate) fn set_rect(&mut self, rect: Rect<u32, PixelSize>) {
        self.root.set_rect(rect, self.border_width())
    }

    pub(crate) fn draw(&mut self, painter: &mut Painter) {
        painter.widget_ctx(self.root.rect().cast(), self.border_color(), false);
        self.root.draw(painter);
    }

    pub(crate) fn active_mut(&mut self) -> &mut TextView {
        self.root.active_mut()
    }

    pub(crate) fn active(&self) -> &TextView {
        self.root.active()
    }

    pub(crate) fn move_cursor_to_point(&mut self, point: Point2D<i32, PixelSize>) {
        self.set_active_cursor_underline(false);
        self.root.move_cursor_to_point(point);
        self.set_active_cursor_block(true);
    }

    pub(crate) fn scroll_views_with_active_acc(
        &mut self,
        acc: Vector2D<f32, PixelSize>,
        duration: Duration,
    ) {
        self.root.scroll_with_active_acc(acc, duration);
    }

    pub(crate) fn split_horizontal(&mut self, buffer: BufferHandle, view_id: BufferViewId) {
        self.set_active_cursor_underline(true);
        take(&mut self.root, |root| {
            root.split(buffer, view_id, SplitDir::Horizontal)
        });
        self.root.set_rect(self.root.rect(), self.border_width());
        self.set_active_cursor_block(true);
    }

    pub(crate) fn split_vertical(&mut self, buffer: BufferHandle, view_id: BufferViewId) {
        self.set_active_cursor_underline(true);
        take(&mut self.root, |root| {
            root.split(buffer, view_id, SplitDir::Vertical)
        });
        self.root.set_rect(self.root.rect(), self.border_width());
        self.set_active_cursor_block(true);
    }

    pub(crate) fn set_left_active(&mut self) {
        self.set_active_cursor_underline(false);
        self.root.set_left_active();
        self.set_active_cursor_block(true);
    }

    pub(crate) fn set_right_active(&mut self) {
        self.set_active_cursor_underline(false);
        self.root.set_right_active();
        self.set_active_cursor_block(true);
    }

    pub(crate) fn set_up_active(&mut self) {
        self.set_active_cursor_underline(false);
        self.root.set_up_active();
        self.set_active_cursor_block(true);
    }

    pub(crate) fn set_down_active(&mut self) {
        self.set_active_cursor_underline(false);
        self.root.set_down_active();
        self.set_active_cursor_block(true);
    }

    pub(crate) fn move_left(&mut self) {
        self.root.move_left();
    }

    pub(crate) fn move_right(&mut self) {
        self.root.move_right();
    }

    pub(crate) fn move_up(&mut self) {
        self.root.move_up();
    }

    pub(crate) fn move_down(&mut self) {
        self.root.move_down();
    }

    pub(crate) fn cycle_prev(&mut self) {
        self.set_active_cursor_underline(false);
        self.root.cycle_prev();
        self.set_active_cursor_block(true);
    }

    pub(crate) fn cycle_next(&mut self) {
        self.set_active_cursor_underline(false);
        self.root.cycle_next();
        self.set_active_cursor_block(true);
    }

    pub(crate) fn grow_active(&mut self) {
        self.root.grow_active();
    }

    pub(crate) fn shrink_active(&mut self) {
        self.root.shrink_active();
    }

    fn border_color(&self) -> Color {
        self.theme_set
            .get(&self.config.borrow().theme)
            .textview
            .border_color
    }

    fn border_width(&self) -> u32 {
        self.theme_set
            .get(&self.config.borrow().theme)
            .textview
            .border_width
    }

    fn set_active_cursor_underline(&mut self, snap_to_cursor: bool) {
        let mut ctx = self.active_mut().edit_ctx();
        ctx.set_cursor_style(CursorStyle::Underline);
        if snap_to_cursor {
            ctx.snap_to_cursor(true);
        }
    }

    fn set_active_cursor_block(&mut self, snap_to_cursor: bool) {
        let mut ctx = self.active_mut().edit_ctx();
        ctx.set_cursor_style(CursorStyle::Block);
        if snap_to_cursor {
            ctx.snap_to_cursor(true);
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SplitDir {
    Horizontal,
    Vertical,
}

enum Node {
    Leaf {
        rect: Rect<u32, PixelSize>,
        view: TextView,
    },
    Inner {
        rect: Rect<u32, PixelSize>,
        children: Vec<Node>,
        active: usize,
        split_dir: SplitDir,
    },
}

impl Node {
    fn new_leaf(rect: Rect<u32, PixelSize>, buffer: BufferHandle, view_id: BufferViewId) -> Node {
        Node::Leaf {
            rect,
            view: TextView::new(rect, buffer, view_id),
        }
    }

    fn is_leaf(&self) -> bool {
        match self {
            Node::Leaf { .. } => true,
            Node::Inner { .. } => false,
        }
    }

    fn split(self, buffer: BufferHandle, view_id: BufferViewId, dir: SplitDir) -> Node {
        match self {
            Node::Leaf { rect, view } => {
                let new = Node::new_leaf(rect, buffer, view_id);
                let old = Node::Leaf { rect, view };
                Node::Inner {
                    rect,
                    active: 0,
                    children: vec![new, old],
                    split_dir: dir,
                }
            }
            Node::Inner {
                rect,
                mut children,
                active,
                split_dir,
            } => {
                let new = if children[active].is_leaf() {
                    if split_dir == dir {
                        Node::new_leaf(rect, buffer, view_id)
                    } else {
                        children.remove(active).split(buffer, view_id, dir)
                    }
                } else {
                    children.remove(active).split(buffer, view_id, dir)
                };
                children.insert(active, new);
                Node::Inner {
                    rect,
                    children,
                    active,
                    split_dir,
                }
            }
        }
    }

    fn set_rect(&mut self, new_rect: Rect<u32, PixelSize>, border_width: u32) {
        match self {
            Node::Leaf { rect, view } => {
                *rect = new_rect;
                view.set_rect(new_rect);
            }
            Node::Inner {
                rect,
                children,
                split_dir,
                ..
            } => {
                *rect = new_rect;
                match split_dir {
                    SplitDir::Horizontal => {
                        let total_height = rect.height();
                        let width = rect.width();
                        let mut origin = rect.origin;
                        let height_per = total_height / children.len() as u32;
                        let extras = total_height as usize % children.len();
                        assert!(border_width < height_per);
                        for i in 0..extras {
                            let height = height_per + 1 - border_width;
                            let rect = Rect::new(origin, size2(width, height));
                            children[i].set_rect(rect, border_width);
                            origin.y += height_per + 1;
                        }
                        for i in extras..children.len() - 1 {
                            let height = height_per - border_width;
                            let rect = Rect::new(origin, size2(width, height));
                            children[i].set_rect(rect, border_width);
                            origin.y += height_per;
                        }
                        children
                            .last_mut()
                            .unwrap()
                            .set_rect(Rect::new(origin, size2(width, height_per)), border_width);
                    }
                    SplitDir::Vertical => {
                        let total_width = rect.width();
                        let height = rect.height();
                        let mut origin = rect.origin;
                        let width_per = total_width / children.len() as u32;
                        let extras = total_width as usize % children.len();
                        assert!(border_width < width_per);
                        for i in 0..extras {
                            let width = width_per + 1 - border_width;
                            let rect = Rect::new(origin, size2(width, height));
                            children[i].set_rect(rect, border_width);
                            origin.x += width_per + 1;
                        }
                        for i in extras..children.len() - 1 {
                            let width = width_per - border_width;
                            let rect = Rect::new(origin, size2(width, height));
                            children[i].set_rect(rect, border_width);
                            origin.x += width_per;
                        }
                        children
                            .last_mut()
                            .unwrap()
                            .set_rect(Rect::new(origin, size2(width_per, height)), border_width);
                    }
                }
            }
        }
    }

    fn draw(&mut self, painter: &mut Painter) {
        match self {
            Node::Leaf { view, .. } => view.draw(painter),
            Node::Inner { children, .. } => {
                for child in children {
                    child.draw(painter)
                }
            }
        }
    }

    fn active_mut(&mut self) -> &mut TextView {
        match self {
            Node::Leaf { view, .. } => view,
            Node::Inner {
                children, active, ..
            } => children[*active].active_mut(),
        }
    }

    fn active(&self) -> &TextView {
        match self {
            Node::Leaf { view, .. } => view,
            Node::Inner {
                children, active, ..
            } => children[*active].active(),
        }
    }

    fn move_cursor_to_point(&mut self, point: Point2D<i32, PixelSize>) {
        if !self.contains_point(point) {
            return;
        }
        match self {
            Node::Leaf { view, .. } => view.move_cursor_to_point(point),
            Node::Inner {
                children, active, ..
            } => {
                for i in 0..children.len() {
                    if children[i].contains_point(point) {
                        *active = i;
                        children[i].move_cursor_to_point(point);
                        return;
                    }
                }
            }
        }
    }

    fn contains_point(&self, point: Point2D<i32, PixelSize>) -> bool {
        match self {
            Node::Leaf { rect, .. } | Node::Inner { rect, .. } => rect.cast().contains(point),
        }
    }

    fn scroll_with_active_acc(&mut self, acc: Vector2D<f32, PixelSize>, duration: Duration) {
        match self {
            Node::Leaf { view, .. } => {
                view.scroll(acc, duration);
            }
            Node::Inner {
                children, active, ..
            } => {
                let active = *active;
                for i in 0..children.len() {
                    if i == active {
                        children[i].scroll_with_active_acc(acc, duration);
                    } else {
                        children[i].scroll_with_active_acc(vec2(0.0, 0.0), duration);
                    }
                }
            }
        }
    }

    fn rect(&self) -> Rect<u32, PixelSize> {
        match self {
            Node::Leaf { rect, .. } | Node::Inner { rect, .. } => *rect,
        }
    }

    fn set_rightmost_active(&mut self) {
        match self {
            Node::Leaf { .. } => {}
            Node::Inner {
                children,
                active,
                split_dir,
                ..
            } => match split_dir {
                SplitDir::Horizontal => children[*active].set_rightmost_active(),
                SplitDir::Vertical => *active = children.len() - 1,
            },
        }
    }

    fn set_leftmost_active(&mut self) {
        match self {
            Node::Leaf { .. } => {}
            Node::Inner {
                children,
                active,
                split_dir,
                ..
            } => match split_dir {
                SplitDir::Horizontal => children[*active].set_leftmost_active(),
                SplitDir::Vertical => *active = 0,
            },
        }
    }

    fn set_topmost_active(&mut self) {
        match self {
            Node::Leaf { .. } => {}
            Node::Inner {
                children,
                active,
                split_dir,
                ..
            } => match split_dir {
                SplitDir::Horizontal => *active = 0,
                SplitDir::Vertical => children[*active].set_leftmost_active(),
            },
        }
    }

    fn set_bottommost_active(&mut self) {
        match self {
            Node::Leaf { .. } => {}
            Node::Inner {
                children,
                active,
                split_dir,
                ..
            } => match split_dir {
                SplitDir::Horizontal => *active = children.len() - 1,
                SplitDir::Vertical => children[*active].set_leftmost_active(),
            },
        }
    }

    fn set_left_active(&mut self) -> bool {
        match self {
            Node::Leaf { .. } => false,
            Node::Inner {
                children,
                active,
                split_dir,
                ..
            } => {
                if children[*active].set_left_active() {
                    return true;
                }
                if *split_dir == SplitDir::Vertical && *active > 0 {
                    *active = *active - 1;
                    children[*active].set_rightmost_active();
                    return true;
                }
                false
            }
        }
    }

    fn set_right_active(&mut self) -> bool {
        match self {
            Node::Leaf { .. } => false,
            Node::Inner {
                children,
                active,
                split_dir,
                ..
            } => {
                if children[*active].set_right_active() {
                    return true;
                }
                if *split_dir == SplitDir::Vertical && *active < children.len() - 1 {
                    *active = *active + 1;
                    children[*active].set_leftmost_active();
                    return true;
                }
                false
            }
        }
    }

    fn set_up_active(&mut self) -> bool {
        match self {
            Node::Leaf { .. } => false,
            Node::Inner {
                children,
                active,
                split_dir,
                ..
            } => {
                if children[*active].set_up_active() {
                    return true;
                }
                if *split_dir == SplitDir::Horizontal && *active > 0 {
                    *active = *active - 1;
                    children[*active].set_bottommost_active();
                    return true;
                }
                false
            }
        }
    }

    fn set_down_active(&mut self) -> bool {
        match self {
            Node::Leaf { .. } => false,
            Node::Inner {
                children,
                active,
                split_dir,
                ..
            } => {
                if children[*active].set_down_active() {
                    return true;
                }
                if *split_dir == SplitDir::Horizontal && *active < children.len() - 1 {
                    *active = *active + 1;
                    children[*active].set_topmost_active();
                    return true;
                }
                false
            }
        }
    }

    fn move_left(&mut self) {
        unimplemented!()
    }

    fn move_right(&mut self) {
        unimplemented!()
    }

    fn move_up(&mut self) {
        unimplemented!()
    }

    fn move_down(&mut self) {
        unimplemented!()
    }

    fn cycle_prev(&mut self) {
        unimplemented!()
    }

    fn cycle_next(&mut self) {
        unimplemented!()
    }

    fn grow_active(&mut self) {
        unimplemented!()
    }

    fn shrink_active(&mut self) {
        unimplemented!()
    }
}
