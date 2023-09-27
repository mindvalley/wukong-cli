use crate::commands::tui::events::key::Key;

pub fn down_event(key: Key) -> bool {
    matches!(key, Key::Down | Key::Char('j'))
}

pub fn up_event(key: Key) -> bool {
    matches!(key, Key::Up | Key::Char('k'))
}

pub fn left_event(key: Key) -> bool {
    matches!(key, Key::Left | Key::Char('h'))
}

pub fn right_event(key: Key) -> bool {
    matches!(key, Key::Right | Key::Char('l'))
}

pub fn back_event(key: Key) -> bool {
    key == Key::Esc
}

pub fn exit_event(key: Key) -> bool {
    key == Key::Char('q')
}
