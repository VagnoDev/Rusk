use crate::app::actions::Action;
use crate::app::mode::Mode;

use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingKey {
    None,
    Space,
    G,
}

pub fn keybinding(key: KeyEvent, mode: &Mode, pending_key: &mut PendingKey) -> Option<Action> {
    match mode {
        Mode::Normal => {
            if *pending_key != PendingKey::None {
                let pending = *pending_key;
                *pending_key = PendingKey::None;

                match (pending, key.code) {
                    (PendingKey::G, KeyCode::Char('e')) => {
                        return Some(Action::MoveToPrevWordEnd);
                    }
                    (PendingKey::G, KeyCode::Char('g')) => {
                        return Some(Action::MoveToStart);
                    }
                    (PendingKey::Space, KeyCode::Char('e')) => {
                        return Some(Action::SetExplorer);
                    }
                    _ => {}
                }
            }
            match key.code {
                KeyCode::Char('h') => Some(Action::MoveLeft),
                KeyCode::Char('l') => Some(Action::MoveRight),
                KeyCode::Char('j') => Some(Action::MoveDown),
                KeyCode::Char('k') => Some(Action::MoveUp),
                KeyCode::Char('a') => Some(Action::InsertForward),
                KeyCode::Char('i') => Some(Action::InserBackward),
                KeyCode::Char(':') => Some(Action::SetCommand),
                KeyCode::Char('0') => Some(Action::StartLine),
                KeyCode::Char('$') => Some(Action::EndLine),
                KeyCode::Char('w') => Some(Action::MoveNext),
                KeyCode::Char('b') => Some(Action::MovePrev),
                KeyCode::Char('e') => Some(Action::MoveToWord),
                KeyCode::Char('G') => Some(Action::MoveToEnd),
                KeyCode::Char(' ') => {
                    *pending_key = PendingKey::Space;
                    None
                }
                KeyCode::Char('g') => {
                    *pending_key = PendingKey::G;
                    None
                },
                _ => None,
            }
        }
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
