use crate::editor::buffer::Buffer;

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
    let line = &buffer.lines[buffer.column];

    while buffer.row < line.len() {
        let Some(c) = line.chars().nth(buffer.row) else {
            break;
        };

        if c == ' ' {
            break;
        }

        buffer.row += 1;
    }
    while buffer.row < line.len() {
        let Some(c) = line.chars().nth(buffer.row) else {
            break;
        };

        if c != ' ' {
            break;
        }

        buffer.row += 1;
    }
}

pub fn move_toprev(buffer: &mut Buffer) {
    let line = &buffer.lines[buffer.column];

    while buffer.row > 0 {
        let Some(c) = line.chars().nth(buffer.row) else {
            break;
        };

        if c == ' ' {
            break;
        }
        if buffer.row > 0 {
            buffer.row -= 1;
        }
    }
    while buffer.row > 0 {
        let Some(c) = line.chars().nth(buffer.row) else {
            break;
        };

        if c == ' ' {
            if buffer.row > 0 {
                buffer.row -= 1;
            }
        } else {
            break;
        }
    }
}