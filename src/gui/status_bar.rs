use crate::app::{
    context::EditorContext,
    messages::{Message, MessageLevel},
    mode::Mode,
};
use crate::editor::buffer::Buffer;
use std::time::{Duration, Instant};

use super::display::{Colors, Display};

pub(super) fn draw_status_bar(display: &mut Display, context: &EditorContext, buffer: &Buffer) {
    let width = display.width as usize;
    let height = display.height as usize;

    if height == 0 || width == 0 {
        return;
    }

    let start = (height - 1) * width;

    let mode_status = match context.mode {
        Mode::Normal => context
            .search_line
            .clone()
            .unwrap_or_else(|| file_info(context, buffer)),
        Mode::Insert => "-- INSERT --".to_string(),
        Mode::Explorer => "-- EXPLORER --".to_string(),
        Mode::Command => {
            if context.savedialog {
                format!("Save as: {}", context.command_buffer)
            } else {
                if context.search_forward.is_some() {
                    let prefix = if context.search_forward == Some(false) {
                        '?'
                    } else {
                        '/'
                    };
                    format!("{prefix}{}", context.command_buffer)
                } else {
                    format!(":{}", context.command_buffer)
                }
            }
        }
    };

    let is_error = matches!(
        context.current_message.as_ref(),
        Some(message)
            if Instant::now() < message.duration
                && matches!(message.level, MessageLevel::Error)
    );

    let status = match context.current_message.as_ref() {
        Some(message) if Instant::now() < message.duration => message.message.clone(),
        _ => mode_status,
    };

    for i in 0..width {
        if let Some(cell) = display.cells.get_mut(start + i) {
            cell.symbol = ' ';
            if is_error {
                cell.bg = Colors::Red;
                cell.fg = Colors::White;
            }
        }
    }

    for (i, ch) in status.chars().take(width).enumerate() {
        if let Some(cell) = display.cells.get_mut(start + i) {
            cell.symbol = ch;
        }
    }

    if matches!(context.mode, Mode::Normal) && !context.pending_keys.is_empty() {
        let pending: Vec<char> = context.pending_keys.chars().collect();
        let len = pending.len().min(width);
        let offset = width - len;
        for (i, ch) in pending.iter().take(len).enumerate() {
            if let Some(cell) = display.cells.get_mut(start + offset + i) {
                cell.symbol = *ch;
            }
        }
    }
}

fn file_info(context: &EditorContext, buffer: &Buffer) -> String {
    let name = context
        .file_path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "[No Name]".to_string());
    let lines = buffer.lines.len();
    let bytes: usize = buffer.lines.iter().map(|line| line.len() + 1).sum();
    format!("\"{name}\" {lines}L, {bytes}B")
}

pub fn send_message(context: &mut EditorContext, message: String, level: MessageLevel) {
    context.current_message = Some(Message {
        message,
        level,
        duration: Instant::now() + Duration::from_secs(2),
    });
}
