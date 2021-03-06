// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use euclid::{point2, size2, Rect, Size2D};
use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::CursorStyle;
use crate::common::{PixelSize, DPI};
use crate::config::Config;
use crate::input::{Action, Motion, MotionOrObj};
use crate::painter::Painter;
use crate::style::TextStyle;
use crate::text::{RopeOrStr, ShapedText, TextAlignment, TextShaper};
use crate::theme::Theme;

const TAB_WIDTH: usize = 8;

pub(crate) struct CmdPrompt {
    command: String,
    pub(crate) rect: Rect<u32, PixelSize>,
    // Text shaping
    dpi: Size2D<u32, DPI>,
    text_shaper: Rc<RefCell<TextShaper>>,
    // Shaped text
    shaped: ShapedText,
    ascender: i32,
    descender: i32,
    // Misc.
    config: Rc<Config>,
    theme: Rc<Theme>,
    prompt_len: usize,
    cursor_bidx: usize,
    cursor_gidx: usize,
}

impl CmdPrompt {
    pub(crate) fn new(
        config: Rc<Config>,
        dpi: Size2D<u32, DPI>,
        text_shaper: Rc<RefCell<TextShaper>>,
        win_rect: Rect<u32, PixelSize>,
        theme: Rc<Theme>,
    ) -> CmdPrompt {
        let (ascender, descender, shaped) = {
            let shaper = &mut *text_shaper.borrow_mut();
            let raster = shaper
                .get_raster(config.prompt_face, TextStyle::default())
                .unwrap();
            let metrics = raster.get_metrics(config.prompt_font_size, dpi);
            let shaped = shaper.shape_line(
                "".into(),
                dpi,
                TAB_WIDTH,
                &[(0, config.prompt_face)],
                &[(0, TextStyle::default())],
                &[(0, config.prompt_font_size)],
                &[(0, theme.prompt.foreground)],
                &[(0, None)],
                &[(0, TextAlignment::Left)],
            );
            (metrics.ascender, metrics.descender, shaped)
        };
        let height = (ascender - descender) as u32;
        let rheight = height + config.prompt_padding_vertical * 2;
        assert!(win_rect.size.height > rheight);
        let rect = Rect::new(
            point2(
                win_rect.origin.x,
                win_rect.origin.y + win_rect.size.height - rheight,
            ),
            size2(win_rect.size.width, rheight),
        );
        CmdPrompt {
            command: "".to_owned(),
            rect,
            config,
            dpi,
            text_shaper,
            shaped,
            ascender,
            descender,
            theme,
            prompt_len: 0,
            cursor_bidx: 0,
            cursor_gidx: 0,
        }
    }

    pub(crate) fn draw(&self, painter: &mut Painter) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut painter = painter.widget_ctx(self.rect.cast(), self.theme.prompt.background, false);
        let pos = point2(
            self.config.prompt_padding_horizontal as i32,
            self.config.prompt_padding_vertical as i32 + self.ascender,
        );
        let cursor = if self.command.len() > 0 {
            Some((
                self.cursor_gidx,
                self.theme.prompt.cursor,
                CursorStyle::Line,
            ))
        } else {
            None
        };
        painter.draw_shaped_text(
            shaper,
            pos,
            &self.shaped,
            cursor,
            self.rect.size.width - self.config.prompt_padding_horizontal,
            (self.ascender - self.descender) as u32,
            false,
        );
    }

    pub(crate) fn resize(&mut self, win_rect: Rect<u32, PixelSize>) -> Rect<u32, PixelSize> {
        let height = (self.ascender - self.descender) as u32;
        let rheight = height + self.config.prompt_padding_vertical * 2;
        assert!(win_rect.size.height > rheight);
        self.rect.origin.x = win_rect.origin.x;
        self.rect.origin.y = win_rect.origin.y + win_rect.size.height - rheight;
        self.rect.size.width = win_rect.size.width;
        Rect::new(
            win_rect.origin,
            size2(win_rect.size.width, win_rect.size.height - rheight),
        )
    }

    pub(crate) fn set_prompt(&mut self, s: &str) {
        self.command.clear();
        self.command.push_str(s);
        self.prompt_len = s.len();
        let (_, gidx) = bidx_gidx_from_bidx(&self.command, self.prompt_len);
        self.cursor_bidx = self.prompt_len;
        self.cursor_gidx = gidx;
        self.reshape();
    }

    pub(crate) fn clear(&mut self) {
        self.command.clear();
        self.prompt_len = 0;
        self.cursor_bidx = 0;
        self.cursor_gidx = 0;
        self.reshape();
    }

    pub(crate) fn get_command(&mut self) -> String {
        self.command[self.prompt_len..].to_owned()
    }

    pub(crate) fn handle_action(&mut self, action: &Action) {
        match action {
            Action::Move(m) => match m {
                MotionOrObj::Motion(Motion::Left(0)) | MotionOrObj::Motion(Motion::Right(0)) => {
                    return
                }
                MotionOrObj::Motion(Motion::Left(mut n)) => {
                    let mut start = self.cursor_bidx;
                    let mut cis = self.command[self.prompt_len..self.cursor_bidx].char_indices();
                    while let Some((i, _)) = cis.next_back() {
                        start = self.prompt_len + i;
                        n -= 1;
                        if n == 0 {
                            break;
                        }
                    }
                    if start == self.cursor_bidx {
                        return;
                    }
                    let (bidx, gidx) = bidx_gidx_from_bidx(&self.command, start);
                    self.cursor_bidx = bidx;
                    self.cursor_gidx = gidx;
                }
                MotionOrObj::Motion(Motion::Right(mut n)) => {
                    n += 1;
                    let mut end = self.cursor_bidx;
                    for (i, _) in self.command[self.cursor_bidx..].char_indices() {
                        end = self.cursor_bidx + i;
                        n -= 1;
                        if n == 0 {
                            break;
                        }
                    }
                    if n > 0 {
                        end = self.command.len();
                    }
                    if end == self.cursor_bidx {
                        return;
                    }
                    let (bidx, gidx) = bidx_gidx_from_bidx(&self.command, end);
                    self.cursor_bidx = bidx;
                    self.cursor_gidx = gidx;
                }
                MotionOrObj::Motion(Motion::LineStart) => {
                    let (bidx, gidx) = bidx_gidx_from_bidx(&self.command, self.prompt_len);
                    self.cursor_bidx = bidx;
                    self.cursor_gidx = gidx;
                }
                MotionOrObj::Motion(Motion::LineEnd) => {
                    let (bidx, gidx) = bidx_gidx_from_bidx(&self.command, self.command.len());
                    self.cursor_bidx = bidx;
                    self.cursor_gidx = gidx;
                }
                _ => {}
            },
            Action::Delete(m) => match m {
                MotionOrObj::Motion(Motion::Left(0)) | MotionOrObj::Motion(Motion::Right(0)) => {
                    return
                }
                MotionOrObj::Motion(Motion::Left(mut n)) => {
                    let mut start = self.cursor_bidx;
                    let mut cis = self.command[self.prompt_len..self.cursor_bidx].char_indices();
                    while let Some((i, _)) = cis.next_back() {
                        start = self.prompt_len + i;
                        n -= 1;
                        if n == 0 {
                            break;
                        }
                    }
                    if start == self.cursor_bidx {
                        return;
                    }
                    self.command.replace_range(start..self.cursor_bidx, "");
                    let (bidx, gidx) = bidx_gidx_from_bidx(&self.command, start);
                    self.cursor_bidx = bidx;
                    self.cursor_gidx = gidx;
                }
                MotionOrObj::Motion(Motion::Right(mut n)) => {
                    n += 1;
                    let mut end = self.cursor_bidx;
                    for (i, _) in self.command[self.cursor_bidx..].char_indices() {
                        end = self.cursor_bidx + i;
                        n -= 1;
                        if n == 0 {
                            break;
                        }
                    }
                    if n > 0 {
                        end = self.command.len();
                    }
                    if end == self.cursor_bidx {
                        return;
                    }
                    self.command.replace_range(self.cursor_bidx..end, "");
                    let (bidx, gidx) = bidx_gidx_from_bidx(&self.command, self.cursor_bidx);
                    self.cursor_bidx = bidx;
                    self.cursor_gidx = gidx;
                }
                MotionOrObj::Motion(Motion::LineStart) => {
                    self.command
                        .replace_range(self.prompt_len..self.cursor_bidx, "");
                    let (bidx, gidx) = bidx_gidx_from_bidx(&self.command, self.prompt_len);
                    self.cursor_bidx = bidx;
                    self.cursor_gidx = gidx;
                }
                MotionOrObj::Motion(Motion::LineEnd) => {
                    self.command.replace_range(self.cursor_bidx.., "");
                    let (bidx, gidx) = bidx_gidx_from_bidx(&self.command, self.cursor_bidx);
                    self.cursor_bidx = bidx;
                    self.cursor_gidx = gidx;
                }
                _ => {}
            },
            Action::InsertChar(c) => {
                self.command.insert(self.cursor_bidx, *c);
                self.cursor_bidx += c.len_utf8();
                let (bidx, gidx) = bidx_gidx_from_bidx(&self.command, self.cursor_bidx);
                self.cursor_bidx = bidx;
                self.cursor_gidx = gidx;
            }
            _ => {}
        }
        self.reshape();
    }

    fn reshape(&mut self) {
        let lc = self.command.chars().count();
        let shaper = &mut *self.text_shaper.borrow_mut();
        self.shaped = shaper.shape_line(
            RopeOrStr::from(self.command.as_ref()),
            self.dpi,
            TAB_WIDTH,
            &[(lc, self.config.prompt_face)],
            &[(lc, TextStyle::default())],
            &[(lc, self.config.prompt_font_size)],
            &[(lc, self.theme.prompt.foreground)],
            &[(lc, None)],
            &[(lc, TextAlignment::Left)],
        );
    }
}

fn bidx_gidx_from_bidx(s: &str, bidx: usize) -> (usize, usize) {
    let (mut gidx, mut blen) = (0, 0);
    for g in s.graphemes(true) {
        let len = g.len();
        if len + blen > bidx {
            return (blen, gidx);
        }
        blen += len;
        if g == "\t" {
            gidx = (gidx / TAB_WIDTH) * TAB_WIDTH + TAB_WIDTH;
        } else {
            gidx += 1;
        }
    }
    (blen, gidx)
}
