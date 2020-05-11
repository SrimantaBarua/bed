// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::{point2, size2, Rect};
use ropey::Rope;

use crate::common::PixelSize;
use crate::painter::WidgetPainter;
use crate::style::Color;
use crate::text::{ShapedText, TextShaper};
use crate::Direction;

pub(super) struct Cursor {
    pub(super) line_num: usize,
    pub(super) line_cidx: usize,
    pub(super) line_gidx: usize,
}

impl Cursor {
    fn default() -> Cursor {
        Cursor {
            line_num: 1,
            line_cidx: 2,
            line_gidx: 2,
        }
    }
}

pub(super) struct BufferView {
    pub(super) rect: Rect<u32, PixelSize>,
    cursor: Cursor,
}

impl BufferView {
    pub(super) fn new(rect: Rect<u32, PixelSize>) -> BufferView {
        BufferView {
            rect: rect,
            cursor: Cursor::default(),
        }
    }

    pub(super) fn move_cursor(&mut self, dirn: Direction, data: &Rope) {
        match dirn {
            Direction::Up => {
                if self.cursor.line_num > 0 {
                    self.cursor.line_num -= 1;
                }
            }
            Direction::Down => {
                if self.cursor.line_num + 1 < data.len_lines() {
                    self.cursor.line_num += 1;
                }
            }
            Direction::Left => {
                if self.cursor.line_gidx > 0 {
                    self.cursor.line_gidx -= 1;
                }
            }
            Direction::Right => self.cursor.line_gidx += 1,
        }
    }

    pub(super) fn draw(
        &self,
        shaped_lines: &[ShapedText],
        shaper: &mut TextShaper,
        painter: &mut WidgetPainter,
    ) {
        let mut pos = point2(0, 0);
        let mut linum = 0;
        let (cline, cgidx) = (self.cursor.line_num, self.cursor.line_gidx);
        for line in shaped_lines {
            pos.y += line.metrics.ascender;
            let mut gidx = 0;

            for (clusters, face, style, size, color, opt_under) in line.styled_iter() {
                for cluster in clusters {
                    if pos.x as u32 >= self.rect.size.width {
                        break;
                    }
                    let raster = shaper.get_raster(face, style).unwrap();
                    let start_x = pos.x;
                    for gi in cluster.glyph_infos {
                        painter.glyph(pos + gi.offset, face, gi.gid, size, color, style, raster);
                        pos.x += gi.advance.width;
                    }
                    let width = pos.x - start_x;
                    if linum == cline && gidx <= cgidx && gidx + cluster.num_graphemes > cgidx {
                        let cwidth = 2;
                        let cheight = line.metrics.ascender - line.metrics.descender;
                        let mut cx = (width * (cgidx - gidx) as i32) / cluster.num_graphemes as i32;
                        cx += start_x;
                        let cy = pos.y - line.metrics.ascender;
                        painter.color_quad(
                            Rect::new(point2(cx, cy), size2(cwidth, cheight)),
                            Color::new(0xff, 0x88, 0x22, 0xff),
                        );
                    }
                    if let Some(under) = opt_under {
                        painter.color_quad(
                            Rect::new(
                                point2(start_x, pos.y - line.metrics.underline_position),
                                size2(width, line.metrics.underline_thickness),
                            ),
                            under,
                        );
                    }
                    gidx += cluster.num_graphemes;
                }
            }
            pos.y -= line.metrics.descender;
            pos.x = 0;
            if pos.y as u32 >= self.rect.size.height {
                break;
            }
            linum += 1;
        }
    }
}
