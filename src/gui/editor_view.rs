use crate::{app::context::EditorContext, editor::buffer::Buffer};

use super::display::Display;

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

    for (y, line) in buffer
        .lines
        .iter()
        .skip(viewport)
        .take(editor_height)
        .enumerate()
    {
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
