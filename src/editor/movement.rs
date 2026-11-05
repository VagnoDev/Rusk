use crate::editor::buffer::Buffer;

#[derive(PartialEq)]
enum CharType {
    Word,
    Punctuation,
    Whitespace,
}

pub fn move_right(buffer: &mut Buffer) {
    let len = buffer.char_count(buffer.column).unwrap_or(0);

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
    let len = lines.line_count();

    if len == 0 || lines.column >= len - 1 {
        return;
    }

    lines.column += 1;

    let line_len = lines.char_count(lines.column).unwrap_or(0);

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

    let line_len = lines.char_count(lines.column).unwrap_or(0);

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
    let line_len = buffer.char_count(buffer.column).unwrap_or(0);

    if line_len == 0 {
        return;
    } else {
        buffer.row = line_len - 1;
    }
}

pub fn move_tonext(buffer: &mut Buffer) {
    if buffer.column >= buffer.line_count() {
        return;
    }

    let Some(current_line) = buffer.line(buffer.column) else {
        return;
    };
    let line: Vec<char> = current_line.chars().collect();

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

    while next_column < buffer.line_count() {
        let Some(next_str) = buffer.line(next_column) else {
            break;
        };
        let next_line: Vec<char> = next_str.chars().collect();

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
    if buffer.line_count() == 0 {
        return;
    }

    if buffer.row == 0 {
        if buffer.column > 0 {
            buffer.column -= 1;
            buffer.row = buffer
                .char_count(buffer.column)
                .unwrap_or(0)
                .saturating_sub(1);
        }
        return;
    }

    let Some(current_line) = buffer.line(buffer.column) else {
        return;
    };
    let line: Vec<char> = current_line.chars().collect();

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

pub fn move_to_next_word(buffer: &mut Buffer) {
    if buffer.line_count() == 0 {
        return;
    }

    if buffer.column >= buffer.line_count() {
        return;
    }

    let mut col = buffer.column;
    let mut pos = buffer.row;

    while col < buffer.line_count() {
        let Some(current_line) = buffer.line(col) else {
            break;
        };
        let line: Vec<char> = current_line.chars().collect();

        if line.is_empty() || pos >= line.len() {
            col += 1;
            pos = 0;
            continue;
        }

        if char_type(line[pos]) == CharType::Word {
            let mut end = pos;

            while end + 1 < line.len() && char_type(line[end + 1]) == CharType::Word {
                end += 1;
            }

            if end > pos {
                buffer.column = col;
                buffer.row = end;
                return;
            }

            pos += 1;
        }

        while pos < line.len() && char_type(line[pos]) != CharType::Word {
            pos += 1;
        }

        if pos < line.len() {
            while pos + 1 < line.len() && char_type(line[pos + 1]) == CharType::Word {
                pos += 1;
            }

            buffer.column = col;
            buffer.row = pos;
            return;
        }

        col += 1;
        pos = 0;
    }
}

pub fn moveto_prev_word_end(buffer: &mut Buffer) {
    if buffer.line_count() == 0 || buffer.column >= buffer.line_count() {
        return;
    }

    let original_row = buffer.row;
    let original_col = buffer.column;

    let mut col = original_col;
    let mut pos = original_row;

    let current_line: Vec<char> = buffer
        .line(col)
        .map_or_else(Vec::new, |l| l.chars().collect());

    if pos < current_line.len() && char_type(current_line[pos]) == CharType::Word {
        while pos > 0 && char_type(current_line[pos - 1]) == CharType::Word {
            pos -= 1;
        }
    }

    loop {
        if pos == 0 {
            if col == 0 {
                break;
            }

            col -= 1;

            let line: Vec<char> = buffer
                .line(col)
                .map_or_else(Vec::new, |l| l.chars().collect());
            pos = line.len();

            if pos == 0 {
                continue;
            }
        }

        let line: Vec<char> = buffer
            .line(col)
            .map_or_else(Vec::new, |l| l.chars().collect());

        while pos > 0 && char_type(line[pos - 1]) != CharType::Word {
            pos -= 1;
        }

        if pos > 0 {
            buffer.column = col;
            buffer.row = pos - 1;
            return;
        }
    }

    buffer.column = original_col;
    buffer.row = original_row;
}

pub fn move_to_start(buffer: &mut Buffer) {
    buffer.row = 0;
    buffer.column = 0;
}

pub fn move_to_end(buffer: &mut Buffer) {
    if buffer.line_count() == 0 {
        return;
    } else {
        buffer.row = 0;
        buffer.column = buffer.line_count().saturating_sub(1);
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
