use std::fs::read_to_string;

use crate::{gui::display_manager::draw, input::Mode};
use crate::context_manager::EditorContext;

#[derive(Debug)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub row: usize,
    pub column: usize,
}

pub fn fetch(file: &str) -> Buffer {
    let mut result = Vec::new();

    for line in read_to_string(file)
        .unwrap() //panic on possible file read errors
        .lines()
    //split the string into a string slices
    {
        result.push(line.to_string());
    }
    Buffer {
        lines: result,
        row: 0,
        column: 0,
    }
}

pub fn move_right(lines: &mut Buffer) {
    let len = lines.lines[lines.column].chars().count();

    if len == 0 || lines.row >= len - 1 {
        return;
    }

    lines.row += 1;
    draw(lines);
}

pub fn move_left(lines: &mut Buffer) {
    if lines.row == 0 {
        return;
    } else {
        lines.row -= 1
    }
    draw(lines);
}

pub fn move_down(lines: &mut Buffer) {
    let len = lines.lines.len();

    if len == 0 || lines.column >= len - 1 {
        return;
    }

    lines.column += 1;

    let line_len = lines.lines[lines.column].chars().count();

    if line_len == 0 {
        lines.row = 0;
    } else if lines.row >= line_len {
        lines.row = line_len - 1;
    }
    draw(lines);
}

pub fn move_up(lines: &mut Buffer) {
    if lines.column == 0 {
        return;
    } else {
        lines.column -= 1
    }
    draw(lines);
}

pub fn insert_char(c: char, buffer: &mut Buffer) {
    let Some(line) = buffer.lines.get_mut(buffer.column) else {
        return;
    };
    line.insert(buffer.row, c);
    buffer.row += 1;

    draw(buffer);
}

pub fn insert_forward(buffer: &mut Buffer, context: &mut EditorContext) {
    buffer.row += 1;

    context.mode = Mode::Insert;

    draw(buffer);
}

pub fn insert_backward(buffer: &mut Buffer, context: &mut EditorContext) {
    if buffer.row == 0 { return; }

    buffer.row -= 1;
    context.mode = Mode::Insert;

    draw(buffer);
}

pub fn set_normal(context: &mut EditorContext) {
    context.mode = Mode::Normal;
}
