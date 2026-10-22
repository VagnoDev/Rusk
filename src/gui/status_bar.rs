use crate::{app::context::EditorContext, app::mode::Mode};

use super::display::Display;

pub(super) fn draw_status_bar(
    display: &mut Display,
    context: &EditorContext,
) {
    let width = display.width as usize;
    let height = display.height as usize;

    if height == 0 || width == 0 {
        return;
    }

    let start = (height - 1) * width;

    let status = match context.mode {
        Mode::Normal => "-- NORMAL --".to_string(),
        Mode::Insert => "-- INSERT --".to_string(),
        Mode::Explorer => "-- EXPLORER --".to_string(),
        Mode::Command => format!(":{}", context.command_buffer),
    };

    for (i, ch) in status.chars().take(width).enumerate() {
        if let Some(cell) = display.cells.get_mut(start + i) {
            cell.symbol = ch;
        }
    }
}