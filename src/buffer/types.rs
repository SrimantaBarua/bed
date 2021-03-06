// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::iter::Peekable;

use ropey::Rope;

use crate::language_client::{
    Diagnostic as LspDiagnostic, DiagnosticCode, DiagnosticRelatedInformation, DiagnosticSeverity,
    DiagnosticTag, Hover as LspHover, HoverContents, Position as LspPosition, Range as LspRange,
};
use crate::theme::Theme;

use super::styled::StyledText;

pub(super) fn internal_to_lsp_position(data: &Rope, line: usize, line_cidx: usize) -> LspPosition {
    let slice = data.line(line).slice(..line_cidx);
    let character = slice.chars().fold(0, |acc, c| acc + c.len_utf16());
    LspPosition { line, character }
}

pub(super) fn internal_cidx_to_lsp_position(data: &Rope, cidx: usize) -> LspPosition {
    let line = data.char_to_line(cidx);
    let slice = data.slice(data.line_to_char(line)..cidx);
    let character = slice.chars().fold(0, |acc, c| acc + c.len_utf16());
    LspPosition { line, character }
}

#[derive(Debug)]
pub(super) struct Hover {
    pub(super) range: Option<Range>,
    pub(super) contents: HoverContents,
}

impl Hover {
    pub(super) fn from(hover: LspHover, data: &Rope) -> Hover {
        Hover {
            range: hover.range.and_then(|range| Range::from(&range, data)),
            contents: hover.contents,
        }
    }
}

#[derive(Debug)]
pub(super) struct Position {
    pub(super) line: usize,
    pub(super) character: usize,
}

impl Position {
    fn from(position: &LspPosition, data: &Rope) -> Option<Position> {
        assert!(position.line < data.len_lines());
        let (mut u8cidx, mut u16cidx) = (0, 0);
        let line = data.line(position.line);
        let mut chars = line.chars();
        while u8cidx < line.len_chars() && u16cidx < position.character {
            let ch = chars.next().unwrap();
            u16cidx += ch.len_utf16();
            u8cidx += 1;
        }
        if u16cidx != position.character {
            None
        } else {
            Some(Position {
                line: position.line,
                character: u8cidx,
            })
        }
    }
}

#[derive(Debug)]
pub(super) struct Range {
    pub(super) start: Position,
    pub(super) end: Position,
}

impl Range {
    fn from(range: &LspRange, data: &Rope) -> Option<Range> {
        Some(Range {
            start: Position::from(&range.start, data)?,
            end: Position::from(&range.end, data)?,
        })
    }
}

#[derive(Debug)]
pub(super) struct Diagnostic {
    pub(super) range: Range,
    pub(super) severity: DiagnosticSeverity,
    pub(super) code: Option<DiagnosticCode>,
    pub(super) source: Option<String>,
    pub(super) message: String,
    pub(super) tags: Option<Vec<DiagnosticTag>>,
    pub(super) related_information: Option<Vec<DiagnosticRelatedInformation>>,
}

impl Diagnostic {
    fn from(diagnostic: &LspDiagnostic, data: &Rope) -> Option<Diagnostic> {
        Some(Diagnostic {
            range: Range::from(&diagnostic.range, data)?,
            severity: diagnostic.severity.clone().unwrap(), // Since we filter out diagnostics which do not have severity
            code: diagnostic.code.clone(),
            source: diagnostic.source.clone(),
            message: diagnostic.message.clone(),
            tags: diagnostic.tags.clone(),
            related_information: diagnostic.relatedInformation.clone(),
        })
    }
}

#[derive(Debug)]
pub(super) struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub(super) fn empty() -> Diagnostics {
        Diagnostics {
            diagnostics: Vec::new(),
        }
    }

    pub(super) fn clear(&mut self) {
        self.diagnostics.clear();
    }

    pub(super) fn set(&mut self, diagnostics: &[LspDiagnostic], data: &Rope) {
        self.diagnostics.clear();
        for diagnostic in diagnostics {
            if let Some(diagnostic) = Diagnostic::from(diagnostic, data) {
                self.diagnostics.push(diagnostic);
            }
        }
    }

    pub(super) fn set_underline(&self, styled_lines: &mut [StyledText], theme: &Theme) {
        let mut line_char_iter = LineCharDiagnosticIter::new(&self.diagnostics);
        let mut next = match line_char_iter.next() {
            Some(x) => x,
            _ => return,
        };
        for i in 0..styled_lines.len() {
            let len_chars = styled_lines[i].unders[styled_lines[i].unders.len() - 1].0;
            styled_lines[i].unders.clear();
            styled_lines[i].unders.push((len_chars, None));
            if i != next.0 {
                continue;
            }
            let under = match next.2.severity {
                DiagnosticSeverity::Warning | DiagnosticSeverity::Hint => {
                    theme.textview.lint_warnings
                }
                DiagnosticSeverity::Error => theme.textview.lint_errors,
                _ => None,
            };
            if under.is_some() {
                if next.1.end > len_chars {
                    next.1.end = len_chars;
                }
                if next.1.start >= len_chars {
                    return;
                }
                styled_lines[i].set_under(next.1.clone(), under);
            }
            next = match line_char_iter.next() {
                Some(x) => x,
                _ => return,
            };
        }
    }

    pub(super) fn lines_chars(&self) -> LineCharDiagnosticIter {
        LineCharDiagnosticIter::new(&self.diagnostics)
    }

    pub(super) fn lines(&self) -> LineIter {
        LineIter {
            peekable: LineCharDiagnosticIter::new(&self.diagnostics).peekable(),
        }
    }
}

pub(super) struct LineHasDiagnostics {
    pub(super) warning: bool,
    pub(super) error: bool,
}

pub(super) struct LineIter<'a> {
    peekable: Peekable<LineCharDiagnosticIter<'a>>,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = (usize, LineHasDiagnostics);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((linum, _, _)) = self.peekable.peek() {
            let linum = *linum;
            let mut ret = LineHasDiagnostics {
                warning: false,
                error: false,
            };
            while let Some((next_linum, _, _)) = self.peekable.peek() {
                if *next_linum != linum {
                    break;
                }
                let (_, _, diag) = self.peekable.next().unwrap();
                match diag.severity {
                    DiagnosticSeverity::Error => ret.error = true,
                    DiagnosticSeverity::Warning | DiagnosticSeverity::Hint => ret.warning = true,
                    _ => {}
                }
            }
            Some((linum, ret))
        } else {
            None
        }
    }
}

pub(super) struct LineCharDiagnosticIter<'a> {
    linum: usize,
    diagnostics: &'a [Diagnostic],
}

impl<'a> LineCharDiagnosticIter<'a> {
    fn new(diagnostics: &'a [Diagnostic]) -> LineCharDiagnosticIter<'a> {
        LineCharDiagnosticIter {
            linum: 0,
            diagnostics,
        }
    }
}

impl<'a> Iterator for LineCharDiagnosticIter<'a> {
    type Item = (usize, std::ops::Range<usize>, &'a Diagnostic);

    fn next(&mut self) -> Option<Self::Item> {
        if self.diagnostics.len() == 0 {
            return None;
        }
        let diag = &self.diagnostics[0];
        if self.linum > diag.range.start.line && self.linum <= diag.range.end.line {
            let linum = self.linum;
            if self.linum < diag.range.end.line {
                self.linum += 1;
                Some((linum, 0..std::usize::MAX, diag))
            } else {
                let range = 0..diag.range.end.character;
                if self.diagnostics.len() > 1 {
                    self.linum = self.diagnostics[1].range.start.line;
                } else {
                    self.linum = std::usize::MAX;
                }
                self.diagnostics = &self.diagnostics[1..];
                Some((linum, range, diag))
            }
        } else {
            let linum = diag.range.start.line;
            assert!(diag.range.end.line >= linum);
            if diag.range.end.line == linum {
                self.diagnostics = &self.diagnostics[1..];
                Some((
                    linum,
                    diag.range.start.character..diag.range.end.character,
                    diag,
                ))
            } else {
                self.linum += 1;
                Some((linum, diag.range.start.character..std::usize::MAX, diag))
            }
        }
    }
}
