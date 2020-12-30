// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::Bed;

impl Bed {
    pub(crate) fn run_command(&mut self, cmd: &str) {
        let mut split = cmd.split(' ');
        match split.next() {
            Some(s) if s == "e" || s == "edit" => {
                let path = &cmd[s.len()..].trim();
                if path.len() == 0 {
                    self.run_edit_command(None)
                } else {
                    self.run_edit_command(Some(path))
                }
            }
            Some("bn") | Some("bnext") if split.next().is_none() => {
                self.text_tree.active_mut().next_view();
                self.redraw_required = true;
            }
            Some("bp") | Some("bprevious") if split.next().is_none() => {
                self.text_tree.active_mut().previous_view();
                self.redraw_required = true;
            }
            Some(s) if s == "sp" || s == "split" => {
                let path = &cmd[s.len()..].trim();
                if path.len() == 0 {
                    self.split_horizontal(None)
                } else {
                    self.split_horizontal(Some(path))
                }
            }
            Some(s) if s == "vsp" || s == "vsplit" => {
                let path = &cmd[s.len()..].trim();
                if path.len() == 0 {
                    self.split_vertical(None)
                } else {
                    self.split_vertical(Some(path))
                }
            }
            _ => {}
        }
    }

    fn run_edit_command(&mut self, optpath: Option<&str>) {
        if let Some(path) = optpath {
            match self.buffer_mgr.read_file(path) {
                Ok(buffer) => {
                    let view_id = self.buffer_mgr.next_view_id();
                    self.text_tree.active_mut().new_view(buffer, view_id);
                    self.redraw_required = true;
                }
                Err(e) => {
                    eprintln!("ERROR: Failed to read path: {}: {}", path, e);
                }
            }
        } else {
            if let Err(e) = self.text_tree.active_mut().reload_buffer() {
                eprintln!("ERROR: Failed to reload buffer: {}", e);
            }
        }
    }

    fn split_horizontal(&mut self, optpath: Option<&str>) {
        if let Some(path) = optpath {
            match self.buffer_mgr.read_file(path) {
                Ok(buffer) => {
                    let view_id = self.buffer_mgr.next_view_id();
                    self.text_tree.split_horizontal(buffer, view_id);
                    self.redraw_required = true;
                }
                Err(e) => {
                    eprintln!("ERROR: Failed to read path: {}: {}", path, e);
                }
            }
        } else {
            let buf_handle = self.text_tree.active().buffer_handle();
            let view_id = self.buffer_mgr.next_view_id();
            self.text_tree.split_horizontal(buf_handle, view_id);
            self.redraw_required = true;
        }
    }

    fn split_vertical(&mut self, optpath: Option<&str>) {
        eprintln!("optpath: {:?}", optpath);
        if let Some(path) = optpath {
            match self.buffer_mgr.read_file(path) {
                Ok(buffer) => {
                    let view_id = self.buffer_mgr.next_view_id();
                    self.text_tree.split_vertical(buffer, view_id);
                    self.redraw_required = true;
                }
                Err(e) => {
                    eprintln!("ERROR: Failed to read path: {}: {}", path, e);
                }
            }
        } else {
            let buf_handle = self.text_tree.active().buffer_handle();
            let view_id = self.buffer_mgr.next_view_id();
            self.text_tree.split_vertical(buf_handle, view_id);
            self.redraw_required = true;
        }
    }
}
