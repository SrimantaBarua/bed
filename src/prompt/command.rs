// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::Bed;

impl Bed {
    pub(crate) fn run_command(&mut self, cmd: &str) {
        match cmd.split(' ').next() {
            Some("e") => {
                let path = &cmd[1..].trim();
                if path.len() == 0 {
                    self.run_edit_command(None)
                } else {
                    self.run_edit_command(Some(path))
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
            // TODO: Reload buffer
        }
    }
}
