use crate::editor::buffer::Buffer;
use crate::app::context::EditorContext;
use crate::app::mode::Mode;

pub fn insert_char(c: char, buffer: &mut Buffer) {
    let Some(line) = buffer.lines.get_mut(buffer.column) else {
        return;
    };
    let byte_index = line
        .char_indices()
        .nth(buffer.row)
        .map(|(index, _)| index)
        .unwrap_or(line.len());
    line.insert(byte_index, c);
    buffer.row += 1;
}

pub fn insert_forward(buffer: &mut Buffer, context: &mut EditorContext) {
    buffer.row += 1;

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
        let Some(line) = buffer.lines.get_mut(buffer.column) else {
            return;
        };

        let index = line
            .char_indices()
            .nth(buffer.row - 1)
            .map(|(index, _)| index)
            .unwrap_or(line.len());

        line.remove(index);
        buffer.row -= 1;
    } else if buffer.column > 0 {
        let current_line = buffer.lines[buffer.column].clone();
        let previous_len = buffer.lines[buffer.column - 1].chars().count();

        {
            let Some(previous) = buffer.lines.get_mut(buffer.column - 1) else {
                return;
            };

            previous.push_str(&current_line);
        }

        buffer.lines.remove(buffer.column);
        buffer.column -= 1;
        buffer.row = previous_len;
    }
}

pub fn insert_newline(buffer: &mut Buffer) {
    let rest = {
        let Some(line) = buffer.lines.get_mut(buffer.column) else {
            return;
        };

        let index = line
            .char_indices()
            .nth(buffer.row)
            .map(|(index, _)| index)
            .unwrap_or(line.len());

        line.split_off(index)
    };

    buffer.lines.insert(buffer.column + 1, rest);
    buffer.column += 1;
    buffer.row = 0;
}
