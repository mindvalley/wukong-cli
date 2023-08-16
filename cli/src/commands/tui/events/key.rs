use std::fmt::Display;

use crossterm::event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    F(u8),
    Char(char),
    Ctrl(char),
    Alt(char),

    Up,
    Down,
    Left,
    Right,

    Esc,
    Enter,
    Tab,
    Backspace,
    Delete,

    Unknown,
}

impl Key {
    pub fn is_exit(&self) -> bool {
        matches!(self, Key::Char('q'))
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Char(c) => write!(f, "<{}>", c),
            Key::Ctrl(c) => write!(f, "<Ctrl+{}>", c),
            Key::Alt(c) => write!(f, "<Alt+{}>", c),
            Key::F(n) => write!(f, "<F{}>", n),
            _ => write!(f, "<{:?}>", self),
        }
    }
}

impl From<event::KeyEvent> for Key {
    fn from(event: event::KeyEvent) -> Self {
        match event {
            event::KeyEvent {
                code: event::KeyCode::Char(c),
                modifiers: event::KeyModifiers::ALT,
                ..
            } => Key::Alt(c),
            event::KeyEvent {
                code: event::KeyCode::Char(c),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => Key::Ctrl(c),
            event::KeyEvent {
                code,
                modifiers: event::KeyModifiers::NONE,
                ..
            } => match code {
                event::KeyCode::Backspace => Key::Backspace,
                event::KeyCode::Enter => Key::Enter,
                event::KeyCode::Left => Key::Left,
                event::KeyCode::Right => Key::Right,
                event::KeyCode::Up => Key::Up,
                event::KeyCode::Down => Key::Down,
                event::KeyCode::Tab => Key::Tab,
                event::KeyCode::Delete => Key::Delete,
                event::KeyCode::F(n) => Key::F(n),
                event::KeyCode::Char(c) => Key::Char(c),
                event::KeyCode::Esc => Key::Esc,
                // event::KeyCode::Home => todo!(),
                // event::KeyCode::End => todo!(),
                // event::KeyCode::PageUp => todo!(),
                // event::KeyCode::PageDown => todo!(),
                // event::KeyCode::BackTab => todo!(),
                // event::KeyCode::Insert => todo!(),
                // event::KeyCode::Null => todo!(),
                // event::KeyCode::CapsLock => todo!(),
                // event::KeyCode::ScrollLock => todo!(),
                // event::KeyCode::NumLock => todo!(),
                // event::KeyCode::PrintScreen => todo!(),
                // event::KeyCode::Pause => todo!(),
                // event::KeyCode::Menu => todo!(),
                // event::KeyCode::KeypadBegin => todo!(),
                // event::KeyCode::Media(_) => todo!(),
                // event::KeyCode::Modifier(_) => todo!(),
                _ => Key::Unknown,
            },
            _ => Key::Unknown,
        }
    }
}
