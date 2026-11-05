use crate::{app::context::EditorContext, OpenBuffer, explorer::tree::Explorer};

use super::display::{Colors, Display};

pub(super) fn draw_explorer(
    display: &mut Display,
    explorer: &Explorer,
    context: &EditorContext,
    buffers: &[OpenBuffer],
) {
    let width = display.width as usize;
    let height = display.height as usize;
    let sidebar_width = 25;

    for (y, entry) in explorer.entries.iter().enumerate() {
        if y >= height.saturating_sub(1) {
            break;
        }

        let is_modified = buffers.iter().any(|open_buffer| {
            open_buffer.path.as_ref() == Some(&entry.path)
                && open_buffer.buffer.modified
        });

        for x in 0..sidebar_width {
            if x >= width {
                break;
            }

            let ch = entry.name.chars().nth(x).unwrap_or(' ');
            let cell_index = y * width + x;

            if let Some(cell) = display.cells.get_mut(cell_index) {
                cell.symbol = ch;

                if is_modified {
                    cell.fg = Colors::YELLOW
                } else {
                    cell.fg = Colors::WHITE;
                }

                if y == explorer.selected {
                    cell.bg = Colors::BLUE;
                } else {
                    cell.bg = Colors::BLACK;
                }
            }
        }
    }

    let separator_x = sidebar_width;

    if separator_x < width {
        for y in 0..height.saturating_sub(1) {
            let cell_index = y * width + separator_x;

            if let Some(cell) = display.cells.get_mut(cell_index) {
                cell.symbol = '|';
                cell.fg = Colors::WHITE;
            }
        }
    }
}
