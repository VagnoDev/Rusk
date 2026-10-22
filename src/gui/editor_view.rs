use crate::editor::buffer::Buffer;

use super::display::Display;

pub(super) fn draw_editor(
    display: &mut Display,
    buffer: &Buffer,
    editor_offset: usize,
) {
    let width = display.width as usize;
    let height = display.height as usize;

    let editor_height = height.saturating_sub(1);

    for (y, line) in buffer.lines.iter().enumerate() {
        if y >= editor_height {
            break;
        }

        for (x, ch) in line.chars().enumerate() {
            let screen_x = editor_offset + x;

            if screen_x >= width {
                break;
            }

            let cell_index = y * width + screen_x;

            if let Some(cell) = display.cells.get_mut(cell_index) {
                cell.symbol = ch;
            }
        }
    }
}