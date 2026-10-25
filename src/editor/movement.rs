use crate::editor::buffer::Buffer;

#[derive(PartialEq)]
enum CharType {
    Word,
    Punctuation,
    Whitespace,
}

pub fn move_right(buffer: &mut Buffer) {
    let Some(line) = buffer.lines.get(buffer.column) else {
        return;
    };

    let len = line.chars().count();

    if len == 0 || buffer.row >= len - 1 {
        return;
    }

    buffer.row += 1;
}

pub fn move_left(lines: &mut Buffer) {
    if lines.row == 0 {
        return;
    } else {
        lines.row -= 1
    }
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
}

pub fn move_up(lines: &mut Buffer) {
    if lines.column == 0 {
        return;
    }

    lines.column -= 1;

    let line_len = lines.lines[lines.column].chars().count();

    if line_len == 0 {
        lines.row = 0;
    } else if lines.row >= line_len {
        lines.row = line_len - 1;
    }
}

pub fn move_tostart(buffer: &mut Buffer) {
    buffer.row = 0;
}

pub fn move_toend(buffer: &mut Buffer) {
    let line_len = buffer.lines[buffer.column].chars().count();

    if line_len == 0 {
        return;
    } else {
        buffer.row = line_len - 1;
    }
}

pub fn move_tonext(buffer: &mut Buffer) {
    if buffer.column >= buffer.lines.len() {
        return;
    }

    let line: Vec<char> = buffer.lines[buffer.column].chars().collect();

    if line.is_empty() || buffer.row >= line.len() {
        return;
    }

    let mut pos = buffer.row;
    let current_type = char_type(line[pos]);

    while pos < line.len() && char_type(line[pos]) == current_type {
        pos += 1;
    }

    while pos < line.len() && char_type(line[pos]) == CharType::Whitespace {
        pos += 1;
    }

    if pos < line.len() {
        buffer.row = pos;
        return;
    }

    let mut next_column = buffer.column + 1;

    while next_column < buffer.lines.len() {
        let next_line: Vec<char> = buffer.lines[next_column].chars().collect();

        if let Some(pos) = next_line
            .iter()
            .position(|&c| char_type(c) != CharType::Whitespace)
        {
            buffer.column = next_column;
            buffer.row = pos;
            return;
        }

        next_column += 1;
    }

    buffer.row = line.len().saturating_sub(1);
}

pub fn move_toprev(buffer: &mut Buffer) {
    if buffer.lines.is_empty() {
        return;
    }

    if buffer.row == 0 {
        if buffer.column > 0 {
            buffer.column -= 1;
            buffer.row = buffer.lines[buffer.column]
                .chars()
                .count()
                .saturating_sub(1);
        }
        return;
    }

    let line: Vec<char> = buffer.lines[buffer.column].chars().collect();

    while buffer.row > 0 && char_type(line[buffer.row - 1]) == CharType::Whitespace {
        buffer.row -= 1;
    }

    if buffer.row == 0 {
        return;
    }

    buffer.row -= 1;
    let current_type = char_type(line[buffer.row]);

    while buffer.row > 0 && char_type(line[buffer.row - 1]) == current_type {
        buffer.row -= 1;
    }
}

fn char_type(c: char) -> CharType {
    if c.is_whitespace() {
        CharType::Whitespace
    } else if c.is_alphanumeric() || c == '_' {
        CharType::Word
    } else {
        CharType::Punctuation
    }
}
