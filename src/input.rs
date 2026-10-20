
use crossterm::{
    event::{KeyCode, KeyEvent},
};


pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    InsertChar(char),
    SetNormal,
    SetCommand,
    EscCommand,
    SendCommand,
    InsertForward,
    InserBackward,
    ExplorerUp,
    ExplorerDown,
    EnterFile,
    SetExplorer,
    Quit,
}

pub enum Mode {
    Normal,
    Insert,
    Command,
    Explorer
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingKey {
    None,
    Space
}

pub fn keybinding(key: KeyEvent, mode: &Mode, pending_key: &mut PendingKey) -> Option<Action> {
    match mode {
        Mode::Normal => {
        match key.code {
            KeyCode::Char('h') => Some(Action::MoveLeft),
            KeyCode::Char('l') => Some(Action::MoveRight),
            KeyCode::Char('j') => Some(Action::MoveDown),
            KeyCode::Char('k') => Some(Action::MoveUp),
            KeyCode::Char('a') => Some(Action::InsertForward),
            KeyCode::Char('i') => Some(Action::InserBackward),
            KeyCode::Char(':') => Some(Action::SetCommand),
            KeyCode::Char(' ') => {
                *pending_key = PendingKey::Space;
                None 
            }
            KeyCode::Char('e') if *pending_key == PendingKey::Space => {
                *pending_key = PendingKey::None;
                Some(Action::SetExplorer)
            }
            _ => None,
        }
        }
        Mode::Insert => {
            match key.code {
                KeyCode::Esc => Some(Action::SetNormal),
                KeyCode::Char(c) => Some(Action::InsertChar(c)),
                _ => None
            }
        }
        Mode::Command => {
            match key.code {
                KeyCode::Esc => Some(Action::EscCommand),
                KeyCode::Char(char) => Some(Action::InsertChar(char)),
                KeyCode::Enter => Some(Action::SendCommand),
                _ => None
            }
        }
        Mode::Explorer => {
            match key.code {
            KeyCode::Up => Some(Action::ExplorerUp),
            KeyCode::Down => Some(Action::ExplorerDown),
            KeyCode::Enter => Some(Action::EnterFile),
            KeyCode::Esc => Some(Action::SetNormal),
            KeyCode::Char('j') => Some(Action::ExplorerDown),
            KeyCode::Char('k') => Some(Action::ExplorerUp),
            _ => None
            }
        }
    }
   
}
