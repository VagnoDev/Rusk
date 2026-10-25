use std::{fs::read_to_string, io::ErrorKind};

#[derive(Debug)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub row: usize,
    pub column: usize,
}

pub fn fetch(file: &str) -> Result<Buffer, std::io::Error> {
    let mut result = Vec::new();

    let content = read_to_string(file);

    match content {
        Ok(content) => {
            for line in content.lines() {
                result.push(line.to_string());
            }
        }
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                result.push(String::new());
            } else {
                return Err(error);
            }
        }
    }

    if result.is_empty() {
        result.push(String::new());
    }

    Ok(Buffer {
        lines: result,
        row: 0,
        column: 0,
    })
}
