// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use glfw::{Key, Modifiers};

use crate::buffer::CursorStyle;

#[derive(Eq, PartialEq)]
pub enum Mode {
    Normal,
    Input,
    Command,
    GPressed(usize),
    DPressed(usize),
}

#[derive(Clone, Copy)]
pub(crate) enum Motion {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    ToLine(usize),
    LineStart,
    LineEnd,
}

pub(crate) enum Action {
    Move(Motion),
    Delete(Motion),
    InsertChar(char),
    UpdateCursorStyle(CursorStyle),
    StartCmdPrompt(String),
    StopCmdPrompt,
    GetCmd,
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
                Key::Up => actions.push(Action::Move(Motion::Up(verb_count))),
                Key::Down => actions.push(Action::Move(Motion::Down(verb_count))),
                Key::Left => actions.push(Action::Move(Motion::Left(verb_count))),
                Key::Right => actions.push(Action::Move(Motion::Right(verb_count))),
                Key::Enter => actions.push(Action::Move(Motion::Down(verb_count))),
                Key::Backspace => actions.push(Action::Move(Motion::Left(verb_count))),
                Key::Home => actions.push(Action::Move(Motion::LineStart)),
                Key::End => actions.push(Action::Move(Motion::LineEnd)),
                // Delete
                Key::Delete => actions.push(Action::Delete(Motion::Right(verb_count))),
                _ => return,
            },
            Mode::Input => match key {
                // Basic movement
                Key::Up => actions.push(Action::Move(Motion::Up(1))),
                Key::Down => actions.push(Action::Move(Motion::Down(1))),
                Key::Left => actions.push(Action::Move(Motion::Left(1))),
                Key::Right => actions.push(Action::Move(Motion::Right(1))),
                Key::Home => actions.push(Action::Move(Motion::LineStart)),
                Key::End => actions.push(Action::Move(Motion::LineEnd)),
                // Insert
                Key::Enter => actions.push(Action::InsertChar('\n')),
                Key::Tab => actions.push(Action::InsertChar('\t')),
                // Delete
                Key::Backspace => actions.push(Action::Delete(Motion::Left(1))),
                Key::Delete => actions.push(Action::Delete(Motion::Right(1))),
                // Exit insert mode
                Key::Escape => {
                    self.mode = Mode::Normal;
                    actions.push(Action::Move(Motion::Left(1)));
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
                Key::Escape => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                }
                _ => return,
            },
            Mode::Command => match key {
                // Basic movement
                Key::Up => actions.push(Action::Move(Motion::Up(1))),
                Key::Down => actions.push(Action::Move(Motion::Down(1))),
                Key::Left => actions.push(Action::Move(Motion::Left(1))),
                Key::Right => actions.push(Action::Move(Motion::Right(1))),
                Key::Home => actions.push(Action::Move(Motion::LineStart)),
                Key::End => actions.push(Action::Move(Motion::LineEnd)),
                // Delete
                Key::Backspace => actions.push(Action::Delete(Motion::Left(1))),
                Key::Delete => actions.push(Action::Delete(Motion::Right(1))),
                // Exit command
                Key::Enter => {
                    self.mode = Mode::Normal;
                    actions.push(Action::GetCmd);
                    actions.push(Action::StopCmdPrompt);
                }
                Key::Escape => {
                    self.mode = Mode::Normal;
                    actions.push(Action::StopCmdPrompt);
                }
                _ => return,
            },
        }
        self.verb_count.clear();
    }

    pub(crate) fn handle_char(&mut self, c: char, actions: &mut Vec<Action>) {
        let verb_count = self.verb_count.parse().unwrap_or(1);
        match self.mode {
            Mode::Normal => match c {
                // Basic movement
                'h' => actions.push(Action::Move(Motion::Left(verb_count))),
                'j' => actions.push(Action::Move(Motion::Down(verb_count))),
                'k' => actions.push(Action::Move(Motion::Up(verb_count))),
                'l' => actions.push(Action::Move(Motion::Right(verb_count))),
                '0' if self.verb_count.len() == 0 => actions.push(Action::Move(Motion::LineStart)),
                '$' => actions.push(Action::Move(Motion::LineEnd)),
                'G' => actions.push(Action::Move(Motion::ToLine(std::usize::MAX))),
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
                    actions.push(Action::Move(Motion::Right(1)));
                }
                'o' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(Action::Move(Motion::LineEnd));
                    actions.push(Action::InsertChar('\n'));
                }
                'O' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(Action::Move(Motion::LineStart));
                    actions.push(Action::InsertChar('\n'));
                    actions.push(Action::Move(Motion::Up(1)));
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
                ':' => {
                    self.mode = Mode::Command;
                    actions.push(Action::StartCmdPrompt(":".to_owned()));
                }
                _ => return,
            },
            Mode::Input => actions.push(Action::InsertChar(c)),
            Mode::GPressed(n) => match c {
                'g' => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                    actions.push(Action::Move(Motion::ToLine(n - 1)));
                }
                _ => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                }
            },
            Mode::DPressed(n) => {
                match c {
                    // Basic movement
                    'h' => actions.push(Action::Delete(Motion::Left(n * verb_count))),
                    'j' => actions.push(Action::Delete(Motion::Down(n * verb_count))),
                    'k' => actions.push(Action::Delete(Motion::Up(n * verb_count))),
                    'l' => actions.push(Action::Delete(Motion::Right(n * verb_count))),
                    '0' if self.verb_count.len() == 0 => {
                        actions.push(Action::Delete(Motion::LineStart))
                    }
                    '$' => actions.push(Action::Delete(Motion::LineEnd)),
                    // Counts
                    c if c.is_ascii_digit() => {
                        self.verb_count.push(c);
                        return;
                    }
                    /*
                    'd' => {
                        self.mode = Mode::Normal;
                        actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                        actions.push(Action::Delete(Motion::Down(n)));
                    }
                    */
                    _ => {}
                }
                self.mode = Mode::Normal;
                actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
            }
            Mode::Command => actions.push(Action::InsertChar(c)),
        }
        self.verb_count.clear();
    }
}
