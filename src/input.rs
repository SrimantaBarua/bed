// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use glfw::{Key, Modifiers};

use crate::buffer::CursorStyle;

pub(crate) enum Mode {
    Normal,
    Input,
    GPressed(usize),
    DPressed(usize),
}

pub(crate) enum Action {
    CursorUp(usize),
    CursorDown(usize),
    CursorLeft(usize),
    CursorRight(usize),
    CursorToLine(usize),
    CursorLineStart,
    CursorLineEnd,
    InsertChar(char),
    DeleteLeft,
    DeleteRight,
    DeleteLinesDown(usize),
    UpdateCursorStyle(CursorStyle),
}

pub(crate) struct State {
    verb_count: String,
    mode: Mode,
}

impl State {
    pub(crate) fn new() -> State {
        State {
            verb_count: String::new(),
            mode: Mode::Normal,
        }
    }

    pub(crate) fn set_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub(crate) fn handle_key(&mut self, key: Key, md: Modifiers, actions: &mut Vec<Action>) {
        let verb_count = self.verb_count.parse().unwrap_or(1);
        match self.mode {
            Mode::Normal => match key {
                // Basic movement
                Key::Up => actions.push(Action::CursorUp(verb_count)),
                Key::Down => actions.push(Action::CursorDown(verb_count)),
                Key::Left => actions.push(Action::CursorLeft(verb_count)),
                Key::Right => actions.push(Action::CursorRight(verb_count)),
                Key::Enter => actions.push(Action::CursorDown(verb_count)),
                Key::Backspace => actions.push(Action::CursorLeft(verb_count)),
                Key::Home => actions.push(Action::CursorLineStart),
                Key::End => actions.push(Action::CursorLineEnd),
                // Delete
                Key::Delete => actions.push(Action::DeleteRight),
                _ => return,
            },
            Mode::Input => match key {
                // Basic movement
                Key::Up => actions.push(Action::CursorUp(1)),
                Key::Down => actions.push(Action::CursorDown(1)),
                Key::Left => actions.push(Action::CursorLeft(1)),
                Key::Right => actions.push(Action::CursorRight(1)),
                Key::Home => actions.push(Action::CursorLineStart),
                Key::End => actions.push(Action::CursorLineEnd),
                // Insert
                Key::Enter => actions.push(Action::InsertChar('\n')),
                Key::Tab => actions.push(Action::InsertChar('\t')),
                // Delete
                Key::Backspace => actions.push(Action::DeleteLeft),
                Key::Delete => actions.push(Action::DeleteRight),
                // Exit insert mode
                Key::Escape => {
                    self.mode = Mode::Normal;
                    actions.push(Action::CursorLeft(1));
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                }
                _ => return,
            },
            Mode::GPressed(_) => match key {
                Key::G => return,
                _ => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                }
            },
            Mode::DPressed(_) => match key {
                Key::D => return,
                _ => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                }
            },
        }
        self.verb_count.clear();
    }

    pub(crate) fn handle_char(&mut self, c: char, actions: &mut Vec<Action>) {
        let verb_count = self.verb_count.parse().unwrap_or(1);
        match self.mode {
            Mode::Normal => match c {
                // Basic movement
                'h' => actions.push(Action::CursorLeft(verb_count)),
                'j' => actions.push(Action::CursorDown(verb_count)),
                'k' => actions.push(Action::CursorUp(verb_count)),
                'l' => actions.push(Action::CursorRight(verb_count)),
                '0' if self.verb_count.len() == 0 => actions.push(Action::CursorLineStart),
                '$' => actions.push(Action::CursorLineEnd),
                'G' => actions.push(Action::CursorToLine(std::usize::MAX)),
                // Counts
                c if c.is_ascii_digit() => {
                    self.verb_count.push(c);
                    return;
                }
                // Enter insert mode. TODO: Proper handling of count
                'i' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                }
                'a' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(Action::CursorRight(1));
                }
                'o' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(Action::CursorLineEnd);
                    actions.push(Action::InsertChar('\n'));
                }
                'O' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(Action::CursorLineStart);
                    actions.push(Action::InsertChar('\n'));
                    actions.push(Action::CursorUp(1));
                }
                // Go into other states
                'g' => {
                    self.mode = Mode::GPressed(verb_count);
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Underline));
                }
                'd' => {
                    self.mode = Mode::DPressed(verb_count);
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Underline));
                }
                _ => return,
            },
            Mode::Input => actions.push(Action::InsertChar(c)),
            Mode::GPressed(n) => match c {
                'g' => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                    actions.push(Action::CursorToLine(n - 1));
                }
                _ => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                }
            },
            Mode::DPressed(n) => match c {
                'd' => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                    actions.push(Action::DeleteLinesDown(n));
                }
                _ => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                }
            },
        }
        self.verb_count.clear();
    }
}
