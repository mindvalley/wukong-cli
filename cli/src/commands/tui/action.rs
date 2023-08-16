use std::{fmt::Display, slice::Iter};

use crossterm::event::KeyCode;

use super::events::key::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    SelectNamespace,
    Quit,
}

impl Action {
    // iterator for enum https://stackoverflow.com/a/21376984
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 2] = [Action::SelectNamespace, Action::Quit];
        ACTIONS.iter()
    }

    pub fn keys(&self) -> &[Key] {
        match self {
            Action::SelectNamespace => &[Key::Char('n')],
            Action::Quit => &[Key::Char('q')],
        }
    }

    pub fn from_key(key: Key) -> Option<Action> {
        Action::iterator()
            .find(|action| action.keys().contains(&key))
            .copied()
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::SelectNamespace => write!(f, "Select namespace"),
            Action::Quit => write!(f, "Quit"),
        }
    }
}
