use std::path::PathBuf;

pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub depth: usize,
}

pub struct Explorer {
    pub current_path: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: usize,
}

pub fn read_dir(path: &PathBuf, depth: usize) -> Explorer {
    let entries = std::fs::read_dir(&path)
        .unwrap_or_else(|e| panic!("cannot read directory {:?}: {}", path, e));
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