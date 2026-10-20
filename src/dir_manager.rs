use std::{path::PathBuf};

pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub depth: usize
}

pub struct Explorer {
    pub current_path: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: usize,
}

pub fn read_dir(path: &PathBuf, depth: usize) -> Explorer {
    let entries = std::fs::read_dir(&path).unwrap();
    let mut vec: Vec<Entry> = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();

        let path = entry.path();

        let file_type = entry.file_type().unwrap();

        let entry = Entry {
                name: name,
                path: path,
                is_dir: file_type.is_dir(),
                expanded: false,
                depth: depth
        };


        vec.push(entry);
    }

    Explorer {
        current_path: (path.clone()),
        entries: (vec),
        selected: (0),
    }
}

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
