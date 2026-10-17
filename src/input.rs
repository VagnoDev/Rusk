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
    SetInsert,
    SetCommand,
    EscCommand,
    InsertCommand,
    SendCommand,
    InsertForward,
    InserBackward,
    Quit,
}

pub enum Mode {
    Normal,
    Insert,
    Command
}

pub fn keybinding(key: KeyEvent, mode: &Mode) -> Option<Action> {
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
                KeyCode::Char(_c) => Some(Action::InsertCommand),
                KeyCode::Enter => Some(Action::SendCommand),
                _ => None
            }
        }
    }
   
}
