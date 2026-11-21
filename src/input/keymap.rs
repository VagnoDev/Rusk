use crate::normal::parser::Key;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn to_key(event: KeyEvent) -> Option<Key> {
    match event.code {
        KeyCode::Esc => Some(Key::Esc),
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::Char(c) if event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Key::Ctrl(c.to_ascii_lowercase()))
        }
        KeyCode::Char(c) => Some(Key::Char(c)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_char() {
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert_eq!(to_key(event), Some(Key::Char('w')));
    }

    #[test]
    fn shifted_char() {
        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        assert_eq!(to_key(event), Some(Key::Char('G')));
    }

    #[test]
    fn ctrl_char() {
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        assert_eq!(to_key(event), Some(Key::Ctrl('f')));
    }

    #[test]
    fn enter_and_esc() {
        assert_eq!(
            to_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some(Key::Enter)
        );
        assert_eq!(
            to_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            Some(Key::Esc)
        );
    }

    #[test]
    fn unmapped() {
        assert_eq!(to_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)), None);
    }
}
