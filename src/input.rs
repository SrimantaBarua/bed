// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use glfw::{Key, Modifiers};

use crate::buffer::CursorStyle;

pub(crate) enum Mode {
    Normal,
    Input,
}

pub(crate) enum Action {
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    CursorLineStart,
    CursorLineEnd,
    InsertChar(char),
    DeleteLeft,
    DeleteRight,
    UpdateCursorStyle(CursorStyle),
}

pub(crate) struct State {
    pub(crate) mode: Mode,
}

impl State {
    pub(crate) fn new() -> State {
        State { mode: Mode::Normal }
    }

    pub(crate) fn set_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub(crate) fn handle_key(&mut self, key: Key, md: Modifiers, actions: &mut Vec<Action>) {
        match self.mode {
            Mode::Normal => match key {
                // Basic movement
                Key::Up => actions.push(Action::CursorUp),
                Key::Down => actions.push(Action::CursorDown),
                Key::Left => actions.push(Action::CursorLeft),
                Key::Right => actions.push(Action::CursorRight),
                Key::Enter => actions.push(Action::CursorDown),
                Key::Backspace => actions.push(Action::CursorLeft),
                Key::Home => actions.push(Action::CursorLineStart),
                Key::End => actions.push(Action::CursorLineEnd),
                // Delete
                Key::Delete => actions.push(Action::DeleteRight),
                _ => {}
            },
            Mode::Input => match key {
                // Basic movement
                Key::Up => actions.push(Action::CursorUp),
                Key::Down => actions.push(Action::CursorDown),
                Key::Left => actions.push(Action::CursorLeft),
                Key::Right => actions.push(Action::CursorRight),
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
                    actions.push(Action::CursorLeft);
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                }
                _ => {}
            },
        }
    }

    pub(crate) fn handle_char(&mut self, c: char, actions: &mut Vec<Action>) {
        match self.mode {
            Mode::Normal => match c {
                // Basic movement
                'h' => actions.push(Action::CursorLeft),
                'j' => actions.push(Action::CursorDown),
                'k' => actions.push(Action::CursorUp),
                'l' => actions.push(Action::CursorRight),
                '0' => actions.push(Action::CursorLineStart),
                '$' => actions.push(Action::CursorLineEnd),
                // Enter insert mode
                'i' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                }
                'a' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(Action::CursorRight);
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
                    actions.push(Action::CursorUp);
                }
                _ => {}
            },
            Mode::Input => actions.push(Action::InsertChar(c)),
        }
    }
}
