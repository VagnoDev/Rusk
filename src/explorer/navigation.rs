use crate::explorer::tree::{read_dir, Explorer};

impl Explorer {
    pub fn explorer_down(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        
        if self.selected < self.entries.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn explorer_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn explorer_toggle(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        if self.entries[self.selected].is_dir && !self.entries[self.selected].expanded {
            let path = self.entries[self.selected].path.clone();
            let depth = self.entries[self.selected].depth;

            let dir = read_dir(&path, depth + 1);
            self.entries.splice(self.selected + 1..self.selected + 1, dir.entries.into_iter());
            self.entries[self.selected].expanded = true;
        } else if self.entries[self.selected].is_dir && self.entries[self.selected].expanded{
            let depth = self.entries[self.selected].depth;
            let mut end = self.selected + 1;
            
            while end < self.entries.len() && self.entries[end].depth > depth {
                end += 1;
            }
            
            self.entries.splice(self.selected + 1..end, []);
            self.entries[self.selected].expanded = false;
        }
    }
}
