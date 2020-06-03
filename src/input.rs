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

#[derive(Clone, Copy)]
pub(crate) enum Object {
    Line,
    Word,
}

#[derive(Clone, Copy)]
pub(crate) enum MotionOrObj {
    Motion(Motion),
    Object(Object),
}

pub(crate) enum Action {
    Move(MotionOrObj),
    Delete(MotionOrObj),
    InsertChar(char),
    UpdateCursorStyle(CursorStyle),
    StartCmdPrompt(String),
    StopCmdPrompt,
    GetCmd,
}

macro_rules! thing {
    // Motions
    (UP, $n:expr) => {
        $crate::input::MotionOrObj::Motion($crate::input::Motion::Up($n))
    };
    (DOWN, $n:expr) => {
        $crate::input::MotionOrObj::Motion($crate::input::Motion::Down($n))
    };
    (LEFT, $n:expr) => {
        $crate::input::MotionOrObj::Motion($crate::input::Motion::Left($n))
    };
    (RIGHT, $n:expr) => {
        $crate::input::MotionOrObj::Motion($crate::input::Motion::Right($n))
    };
    (TO_LINE, $n:expr) => {
        $crate::input::MotionOrObj::Motion($crate::input::Motion::ToLine($n))
    };
    (LINE_START) => {
        $crate::input::MotionOrObj::Motion($crate::input::Motion::LineStart)
    };
    (LINE_END) => {
        $crate::input::MotionOrObj::Motion($crate::input::Motion::LineEnd)
    };
    (LINE) => {
        $crate::input::MotionOrObj::Object($crate::input::Object::Line)
    };
    (WORD) => {
        $crate::input::MotionOrObj::Object($crate::input::Object::Word)
    };
}

macro_rules! act {
    (MOV, $th:ident) => {
        $crate::input::Action::Move(thing!($th))
    };
    (MOV, $th:ident, $n:expr) => {
        $crate::input::Action::Move(thing!($th, $n))
    };
    (DEL, $th:ident) => {
        $crate::input::Action::Delete(thing!($th))
    };
    (DEL, $th:ident, $n:expr) => {
        $crate::input::Action::Delete(thing!($th, $n))
    };
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
                Key::Up => actions.push(act!(MOV, UP, 1)),
                Key::Down => actions.push(act!(MOV, DOWN, 1)),
                Key::Left => actions.push(act!(MOV, LEFT, 1)),
                Key::Right => actions.push(act!(MOV, RIGHT, 1)),
                Key::Enter => actions.push(act!(MOV, DOWN, 1)),
                Key::Backspace => actions.push(act!(MOV, LEFT, 1)),
                Key::Home => actions.push(act!(MOV, LINE_START)),
                Key::End => actions.push(act!(MOV, LINE_END)),
                // Delete
                Key::Delete => actions.push(act!(DEL, RIGHT, 1)),
                _ => return,
            },
            Mode::Input => match key {
                // Basic movement
                Key::Up => actions.push(act!(MOV, UP, 1)),
                Key::Down => actions.push(act!(MOV, DOWN, 1)),
                Key::Left => actions.push(act!(MOV, LEFT, 1)),
                Key::Right => actions.push(act!(MOV, RIGHT, 1)),
                Key::Home => actions.push(act!(MOV, LINE_START)),
                Key::End => actions.push(act!(MOV, LINE_END)),
                // Insert
                Key::Enter => actions.push(Action::InsertChar('\n')),
                Key::Tab => actions.push(Action::InsertChar('\t')),
                // Delete
                Key::Backspace => actions.push(act!(DEL, LEFT, 1)),
                Key::Delete => actions.push(act!(DEL, RIGHT, 1)),
                // Exit insert mode
                Key::Escape => {
                    self.mode = Mode::Normal;
                    actions.push(act!(MOV, LEFT, 1));
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
                Key::Up => actions.push(act!(MOV, UP, 1)),
                Key::Down => actions.push(act!(MOV, DOWN, 1)),
                Key::Left => actions.push(act!(MOV, LEFT, 1)),
                Key::Right => actions.push(act!(MOV, RIGHT, 1)),
                Key::Home => actions.push(act!(MOV, LINE_START)),
                Key::End => actions.push(act!(MOV, LINE_END)),
                // Delete
                Key::Backspace => actions.push(act!(DEL, LEFT, 1)),
                Key::Delete => actions.push(act!(DEL, RIGHT, 1)),
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
                'h' => actions.push(act!(MOV, LEFT, verb_count)),
                'j' => actions.push(act!(MOV, DOWN, verb_count)),
                'k' => actions.push(act!(MOV, UP, verb_count)),
                'l' => actions.push(act!(MOV, RIGHT, verb_count)),
                '0' if self.verb_count.len() == 0 => actions.push(act!(MOV, LINE_START)),
                '$' => actions.push(act!(MOV, LINE_END)),
                'G' => actions.push(act!(MOV, TO_LINE, std::usize::MAX)),
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
                'I' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(act!(MOV, LINE_START));
                }
                'a' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(act!(MOV, RIGHT, 1));
                }
                'A' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(act!(MOV, LINE_END));
                }
                'o' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(act!(MOV, LINE_END));
                    actions.push(Action::InsertChar('\n'));
                }
                'O' => {
                    self.mode = Mode::Input;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Line));
                    actions.push(act!(MOV, LINE_START));
                    actions.push(Action::InsertChar('\n'));
                    actions.push(act!(MOV, UP, 1));
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
                    actions.push(act!(MOV, TO_LINE, n - 1));
                }
                _ => {
                    self.mode = Mode::Normal;
                    actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                }
            },
            Mode::DPressed(n) => {
                match c {
                    // Basic movement
                    'h' => actions.push(act!(DEL, LEFT, n * verb_count)),
                    'j' => actions.push(act!(DEL, DOWN, n * verb_count)),
                    'k' => actions.push(act!(DEL, UP, n * verb_count)),
                    'l' => actions.push(act!(DEL, RIGHT, n * verb_count)),
                    '0' if self.verb_count.len() == 0 => actions.push(act!(DEL, LINE_START)),
                    '$' => actions.push(act!(DEL, LINE_END)),
                    // Counts
                    c if c.is_ascii_digit() => {
                        self.verb_count.push(c);
                        return;
                    }
                    'd' => {
                        self.mode = Mode::Normal;
                        actions.push(Action::UpdateCursorStyle(CursorStyle::Block));
                        actions.push(act!(DEL, DOWN, n));
                    }
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
