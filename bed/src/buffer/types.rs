// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::Ordering;

use language_client::{
    Diagnostic as LCDiagnostic, DiagnosticCode, DiagnosticRelatedInformation, DiagnosticSeverity,
    DiagnosticTag, Position as LCPosition, Range as LCRange,
};
use ropey::Rope;

use crate::theme::Theme;

use super::styled::StyledText;

#[derive(Debug)]
pub(super) struct Position {
    pub(super) line: usize,
    pub(super) character: usize,
}

impl Position {
    fn from(position: LCPosition, data: &Rope) -> Position {
        assert!(position.line < data.len_lines());
        let (mut u8cidx, mut u16cidx) = (0, 0);
        let line = data.line(position.line);
        let mut chars = line.chars();
        while u8cidx < line.len_chars() && u16cidx < position.character {
            let ch = chars.next().unwrap();
            u16cidx += ch.len_utf16();
            u8cidx += 1;
        }
        assert!(u16cidx == position.character);
        Position {
            line: position.line,
            character: u8cidx,
        }
    }
}

#[derive(Debug)]
pub(super) struct Range {
    pub(super) start: Position,
    pub(super) end: Position,
}

impl Range {
    fn from(range: LCRange, data: &Rope) -> Range {
        Range {
            start: Position::from(range.start, data),
            end: Position::from(range.end, data),
        }
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
    fn from(diagnostic: LCDiagnostic, data: &Rope) -> Diagnostic {
        Diagnostic {
            range: Range::from(diagnostic.range, data),
            severity: diagnostic.severity.unwrap(), // Since we filter out diagnostics which do not have severity
            code: diagnostic.code,
            source: diagnostic.source,
            message: diagnostic.message,
            tags: diagnostic.tags,
            related_information: diagnostic.relatedInformation,
        }
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

    pub(super) fn set(&mut self, mut diagnostics: Vec<LCDiagnostic>, data: &Rope) {
        diagnostics.retain(|x| x.severity.is_some());
        diagnostics.sort_by(|a, b| {
            let cmp1 = a.range.start.cmp(&b.range.start);
            if cmp1 == Ordering::Equal {
                a.range.end.cmp(&b.range.end)
            } else {
                cmp1
            }
        });
        self.diagnostics.clear();
        for diagnostic in diagnostics {
            self.diagnostics.push(Diagnostic::from(diagnostic, data));
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
                DiagnosticSeverity::Warning => theme.textview.underline_warnings,
                DiagnosticSeverity::Error => theme.textview.underline_errors,
                _ => None,
            };
            if under.is_some() {
                styled_lines[i].set_under(next.1.clone(), under);
            }
            next = match line_char_iter.next() {
                Some(x) => x,
                _ => return,
            };
        }
    }
}

struct LineCharDiagnosticIter<'a> {
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
