use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,
    Enter,
    Back,
    Refresh,
    Quit,
    None,
}

pub fn map_key(key: KeyEvent) -> Action {
    match (key.code, key.modifiers) {
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => Action::Up,
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => Action::Down,
        (KeyCode::PageUp, _) | (KeyCode::Char('u'), KeyModifiers::CONTROL) => Action::PageUp,
        (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => Action::PageDown,
        (KeyCode::Char('g'), KeyModifiers::NONE) => Action::Top,
        (KeyCode::Char('G'), KeyModifiers::SHIFT) => Action::Bottom,
        (KeyCode::Right, _) | (KeyCode::Enter, _) | (KeyCode::Char('l'), KeyModifiers::NONE) => {
            Action::Enter
        }
        (KeyCode::Left, _) | (KeyCode::Esc, _) | (KeyCode::Char('h'), KeyModifiers::NONE) => {
            Action::Back
        }
        (KeyCode::Char('r'), KeyModifiers::NONE) => Action::Refresh,
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            Action::Quit
        }
        _ => Action::None,
    }
}
