use crate::app::actions::Action;
use crate::app::mode::Mode;

use crate::input::keymap::to_key;
use crossterm::event::{KeyCode, KeyEvent};

pub fn keybinding(key: KeyEvent, mode: &Mode) -> Option<Action> {
    match mode {
        Mode::Normal => to_key(key).map(Action::NormalKey),
        Mode::Insert => match key.code {
            KeyCode::Esc => Some(Action::SetNormal),
            KeyCode::Char(c) => Some(Action::InsertChar(c)),
            KeyCode::Backspace => Some(Action::RemoveChar),
            KeyCode::Enter => Some(Action::InserNewLine),
            _ => None,
        },
        Mode::Command => match key.code {
            KeyCode::Esc => Some(Action::EscCommand),
            KeyCode::Char(char) => Some(Action::InsertChar(char)),
            KeyCode::Backspace => Some(Action::RemoveChar),
            KeyCode::Enter => Some(Action::SendCommand),
            _ => None,
        },
        Mode::Explorer => match key.code {
            KeyCode::Up => Some(Action::ExplorerUp),
            KeyCode::Down => Some(Action::ExplorerDown),
            KeyCode::Enter => Some(Action::EnterFile),
            KeyCode::Esc => Some(Action::SetNormal),
            KeyCode::Char('j') => Some(Action::ExplorerDown),
            KeyCode::Char('k') => Some(Action::ExplorerUp),
            _ => None,
        },
    }
}
