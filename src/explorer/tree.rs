use std::path::PathBuf;

pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub depth: usize,
}

pub struct Explorer {
    pub entries: Vec<Entry>,
    pub selected: usize,
}

pub fn read_dir(path: &PathBuf, depth: usize) -> Explorer {
    let entries = std::fs::read_dir(path)
        .unwrap_or_else(|e| panic!("cannot read directory {:?}: {}", path, e));
    let mut vec: Vec<Entry> = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();

        let path = entry.path();

        let file_type = entry.file_type().unwrap();

        let entry = Entry {
            name,
            path,
            is_dir: file_type.is_dir(),
            expanded: false,
            depth,
        };

        vec.push(entry);
    }

    vec.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Explorer {
        entries: (vec),
        selected: (0),
    }
}
