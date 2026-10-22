use std::fs::read_to_string;

#[derive(Debug)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub row: usize,
    pub column: usize,
}

pub fn fetch(file: &str) -> Buffer {
    let mut result = Vec::new();

    for line in read_to_string(file)
        .unwrap()
        .lines()
    {
        result.push(line.to_string());
    }

    if result.is_empty() {
        result.push(String::new());
    }

    Buffer {
        lines: result,
        row: 0,
        column: 0,
    }
}

