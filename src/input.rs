// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use glfw::{Key, Modifiers};

pub(crate) enum Mode {
    Normal,
    Input,
}

pub(crate) enum Action {
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    InsertChar(char),
    DeleteLeft,
    DeleteRight,
    UpdateCursorStyle,
    None,
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

    pub(crate) fn handle_key(&mut self, key: Key, md: Modifiers) -> Action {
        match self.mode {
            Mode::Normal => match key {
                Key::Up => Action::CursorUp,
                Key::Down => Action::CursorDown,
                Key::Left => Action::CursorLeft,
                Key::Right => Action::CursorRight,
                Key::Enter => Action::CursorDown,
                Key::Backspace => Action::CursorLeft,
                Key::Delete => Action::DeleteRight,
                _ => Action::None,
            },
            Mode::Input => match key {
                Key::Up => Action::CursorUp,
                Key::Down => Action::CursorDown,
                Key::Left => Action::CursorLeft,
                Key::Right => Action::CursorRight,
                Key::Enter => Action::InsertChar('\n'),
                Key::Tab => Action::InsertChar('\t'),
                Key::Backspace => Action::DeleteLeft,
                Key::Delete => Action::DeleteRight,
                Key::Escape => {
                    self.mode = Mode::Normal;
                    Action::UpdateCursorStyle
                }
                _ => Action::None,
            },
        }
    }

    pub(crate) fn handle_char(&mut self, c: char) -> Action {
        match self.mode {
            Mode::Normal => match c {
                'h' => Action::CursorLeft,
                'j' => Action::CursorDown,
                'k' => Action::CursorUp,
                'l' => Action::CursorRight,
                'i' => {
                    self.mode = Mode::Input;
                    Action::UpdateCursorStyle
                }
                _ => Action::None,
            },
            Mode::Input => Action::InsertChar(c),
        }
    }
}
