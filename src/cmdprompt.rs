// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::rc::Rc;

use euclid::{point2, size2, Rect, Size2D};

use crate::common::{PixelSize, DPI};
use crate::font::FaceKey;
use crate::painter::Painter;
use crate::style::{TextSize, TextStyle};
use crate::text::{ShapedText, TextShaper};
use crate::theme::Theme;

const TAB_WIDTH: usize = 8;
const VPAD: u32 = 4;
const HPAD: u32 = 4;

pub(crate) struct CmdPrompt {
    command: String,
    pub(crate) rect: Rect<u32, PixelSize>,
    // Text shaping
    face_key: FaceKey,
    text_size: TextSize,
    dpi: Size2D<u32, DPI>,
    text_shaper: Rc<RefCell<TextShaper>>,
    // Shaped text
    shaped: ShapedText,
    ascender: i32,
    descender: i32,
    // Misc.
    theme: Rc<Theme>,
}

impl CmdPrompt {
    pub(crate) fn new(
        face_key: FaceKey,
        text_size: TextSize,
        dpi: Size2D<u32, DPI>,
        text_shaper: Rc<RefCell<TextShaper>>,
        win_rect: Rect<u32, PixelSize>,
        theme: Rc<Theme>,
    ) -> CmdPrompt {
        let (ascender, descender, shaped) = {
            let shaper = &mut *text_shaper.borrow_mut();
            let raster = shaper.get_raster(face_key, TextStyle::default()).unwrap();
            let metrics = raster.get_metrics(text_size, dpi);
            let shaped = shaper.shape_line(
                ":".into(),
                dpi,
                TAB_WIDTH,
                &[(1, face_key)],
                &[(1, TextStyle::default())],
                &[(1, text_size)],
                &[(1, theme.prompt.foreground)],
                &[(1, None)],
            );
            (metrics.ascender, metrics.descender, shaped)
        };
        let height = (ascender - descender) as u32;
        let rheight = height + VPAD * 2;
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
            rect: rect,
            face_key: face_key,
            text_size: text_size,
            dpi: dpi,
            text_shaper: text_shaper,
            shaped: shaped,
            ascender: ascender,
            descender: descender,
            theme: theme,
        }
    }

    pub(crate) fn draw(&self, painter: &mut Painter) {
        let shaper = &mut *self.text_shaper.borrow_mut();
        let mut painter = painter.widget_ctx(self.rect.cast(), self.theme.prompt.background);
        let pos = point2(HPAD as i32, VPAD as i32 + self.ascender);
        painter.draw_shaped_text(shaper, pos, &self.shaped, None, self.rect.size.width - HPAD);
    }

    pub(crate) fn resize(&mut self, win_rect: Rect<u32, PixelSize>) -> Rect<u32, PixelSize> {
        let height = (self.ascender - self.descender) as u32;
        let rheight = height + VPAD * 2;
        assert!(win_rect.size.height > rheight);
        self.rect.origin.x = win_rect.origin.x;
        self.rect.origin.y = win_rect.origin.y + win_rect.size.height - rheight;
        self.rect.size.width = win_rect.size.width;
        Rect::new(
            win_rect.origin,
            size2(win_rect.size.width, win_rect.size.height - rheight),
        )
    }
}
