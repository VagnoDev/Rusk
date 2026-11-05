use crate::app::context::EditorContext;
use crate::app::mode::Mode;
use crate::editor::buffer::Buffer;

pub fn insert_char(c: char, buffer: &mut Buffer) {
    let modified = buffer.insert_char_at(buffer.row, buffer.column, c);
    if modified {
        buffer.row += 1;
        buffer.mark_modified();
    } else {
        return;
    }
}

pub fn insert_forward(buffer: &mut Buffer, context: &mut EditorContext) {
    let line_len = buffer.char_count(buffer.column).unwrap_or(0);

    if line_len > 0 && buffer.row < line_len {
        buffer.row += 1;
    }

    context.mode = Mode::Insert;
}
pub fn insert_backward(buffer: &mut Buffer, context: &mut EditorContext) {
    if buffer.row == 0 {
        return;
    }

    buffer.row -= 1;
    context.mode = Mode::Insert;
}

pub fn remove_char(buffer: &mut Buffer) {
    if buffer.row > 0 {
        let modified = buffer.remove_char_at(buffer.row - 1, buffer.column);
        if modified.is_some() {
            buffer.row -= 1;
            buffer.mark_modified();
        } else {
            return;
        }
    } else if buffer.column > 0 {
        let Some(current_line) = buffer.line(buffer.column) else {
            return;
        };
        let current_line = current_line.to_string();
        
        let previous_len = buffer.char_count(buffer.column - 1).unwrap_or(0);

        {
            let Some(previous) = buffer.line_mut(buffer.column - 1) else {
                return;
            };

            previous.push_str(&current_line);
        }

        buffer.lines.remove(buffer.column);
        buffer.column -= 1;
        buffer.row = previous_len;
        buffer.mark_modified();
    }
}

pub fn insert_newline(buffer: &mut Buffer) {
    let row = buffer.row;
    let rest = {
        let Some(line) = buffer.line_mut(buffer.column) else {
            return;
        };

        let index = line
            .char_indices()
            .nth(row)
            .map(|(index, _)| index)
            .unwrap_or(line.len());

        line.split_off(index)
    };

    buffer.lines.insert(buffer.column + 1, rest);
    buffer.column += 1;
    buffer.row = 0;
    buffer.mark_modified();
}
