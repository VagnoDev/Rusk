use std::{fs::read_to_string, io::ErrorKind};

use crate::editor::buffer;

#[derive(Debug)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub row: usize,
    pub column: usize,
    pub modified: bool,
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
        modified: false,
    })
}

impl Buffer {
    pub fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|line| line.as_str())
    }

    pub fn line_mut(&mut self, index: usize) -> Option<&mut String> {
        self.lines.get_mut(index)
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn char_at(&self, row: usize, column: usize) -> Option<char> {
        let Some(line) = self.line(column) else {
            return None;
        };

        line.chars().nth(row)
    }

    pub fn insert_char_at(&mut self, row: usize, column: usize, c: char) -> bool {
        let Some(line) = self.line_mut(column) else {
            return false;
        };
        let byte_index = line
            .char_indices()
            .nth(row)
            .map(|(index, _)| index)
            .unwrap_or(line.len());
        line.insert(byte_index, c);

        true
    }

    pub fn char_count(&self, row: usize) -> Option<usize> {
        let Some(line) = self.line(row) else {
            return None;
        };

        return Some(line.chars().count());
    }

    pub fn remove_char_at(&mut self, row: usize, column: usize) -> Option<char> {
        let Some(line) = self.line_mut(column) else {
            return None;
        };

        let Some(index) = line.char_indices().nth(row).map(|(index, _)| index) else {
            return None;
        };

        Some(line.remove(index))
    }

    pub fn mark_modified(&mut self) {
        self.modified = true;
    }

    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
}
