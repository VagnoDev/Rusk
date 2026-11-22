use crate::{app::context::EditorContext, editor::buffer::Buffer};

use super::display::{Colors, Display};

pub(super) fn draw_editor(
    display: &mut Display,
    buffer: &Buffer,
    editor_offset: usize,
    context: &EditorContext,
) {
    let width = display.width as usize;
    let height = display.height as usize;
    let viewport = context.viewport_top;

    let editor_height = height.saturating_sub(1);
    let gutter = gutter_width(buffer);

    for y in 0..editor_height {
        let line_index = viewport + y;
        let row_start = y * width;

        let Some(line) = buffer.lines.get(line_index) else {
            if let Some(cell) = display.cells.get_mut(row_start + editor_offset) {
                cell.symbol = '~';
                cell.fg = Colors::Rgb(69, 71, 90);
            }
            continue;
        };

        let is_current = line_index == buffer.column;
        let number = format!("{:>w$} ", line_index + 1, w = gutter - 1);

        for (x, ch) in number.chars().enumerate() {
            let screen_x = editor_offset + x;

            if screen_x >= width {
                break;
            }

            if let Some(cell) = display.cells.get_mut(row_start + screen_x) {
                cell.symbol = ch;
                cell.fg = if is_current {
                    Colors::Rgb(249, 226, 175)
                } else {
                    Colors::Rgb(88, 91, 112)
                };
            }
        }

        for (x, ch) in line.chars().enumerate() {
            let screen_x = editor_offset + gutter + x;

            if screen_x >= width {
                break;
            }

            let cell_index = row_start + screen_x;

            if let Some(cell) = display.cells.get_mut(cell_index) {
                cell.symbol = ch;
            }
        }
    }
}

pub(super) fn gutter_width(buffer: &Buffer) -> usize {
    buffer.lines.len().to_string().len().max(3) + 1
}

pub fn update_viewport(buffer: &Buffer, context: &mut EditorContext, height: usize) {
    if height == 0 {
        return;
    }
    if buffer.column < context.viewport_top {
        context.viewport_top = buffer.column;
    }
    if buffer.column >= context.viewport_top + height {
        context.viewport_top = buffer.column - height + 1;
    }
}
