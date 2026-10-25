use crate::app::{
    context::EditorContext,
    messages::{Message, MessageLevel},
    mode::Mode,
};
use std::time::{Duration, Instant};

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

    let mode_status = match context.mode {
        Mode::Normal => "-- NORMAL --".to_string(),
        Mode::Insert => "-- INSERT --".to_string(),
        Mode::Explorer => "-- EXPLORER --".to_string(),
        Mode::Command => {
            if context.savedialog {
                format!("Save as: {}", context.command_buffer)
            } else {
                format!(":{}", context.command_buffer)
            }
        },
    };

    let status = match context.current_message.as_ref() {
        Some(message) if Instant::now() < message.duration => {
            message.message.clone()
        }
        _ => mode_status,
    };

    for i in 0..width {
        if let Some(cell) = display.cells.get_mut(start + i) {
            cell.symbol = ' ';
        }
    }

    for (i, ch) in status.chars().take(width).enumerate() {
        if let Some(cell) = display.cells.get_mut(start + i) {
            cell.symbol = ch;
        }
    }
}

pub fn send_message(
    context: &mut EditorContext,
    message: String,
    level: MessageLevel,
) {
    context.current_message = Some(Message {
        message,
        level,
        duration: Instant::now() + Duration::from_secs(3),
    });
}