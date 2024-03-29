// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::Bed;

impl Bed {
    pub(crate) fn handle_command(&mut self, cmd: &str) {
        let mut bytes = cmd.bytes();
        match bytes.next() {
            Some(b'b') => self.handle_b(cmd),
            Some(b'c') => self.handle_c(cmd),
            Some(b'e') => self.handle_e(cmd),
            Some(b'q') => self.handle_q(cmd),
            Some(b's') => self.handle_s(cmd),
            Some(b'v') => self.handle_v(cmd),
            Some(b'w') => self.handle_w(cmd),
            _ => {}
        }
    }

    fn handle_b(&mut self, s: &str) {
        match &s[1..] {
            "n" | "next" => self.textview_tree.active_mut().next_buffer(),
            "p" | "previous" => self.textview_tree.active_mut().prev_buffer(),
            _ => {}
        }
    }

    fn handle_c(&mut self, s: &str) {
        let mut sp = s.split_whitespace();
        match sp.next() {
            Some("cd") => self.change_directory(sp.next()),
            _ => {}
        }
    }

    fn handle_e(&mut self, s: &str) {
        let mut sp = s.split_whitespace();
        match sp.next() {
            Some("e") | Some("edit") => self.load_buffer(sp.next()),
            _ => {}
        }
    }

    fn handle_s(&mut self, s: &str) {
        let mut sp = s.split_whitespace();
        match sp.next() {
            Some("sp") | Some("split") => self.horizontal_split(sp.next()),
            _ => {}
        }
    }

    fn handle_q(&mut self, s: &str) {
        match &s[1..] {
            "" | "uit" => self.window.set_should_close(),
            _ => {}
        }
    }

    fn handle_v(&mut self, s: &str) {
        let mut sp = s.split_whitespace();
        match sp.next() {
            Some("vsp") | Some("vsplit") => self.vertical_split(sp.next()),
            _ => {}
        }
    }

    fn handle_w(&mut self, s: &str) {
        let mut sp = s.split_whitespace();
        match sp.next() {
            Some("w") | Some("write") => self.write_buffer(sp.next()),
            _ => {}
        }
    }
}
