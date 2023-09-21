use std::{fmt::Display, slice::Iter};

use super::events::key::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    OpenNamespaceSelection,
    OpenVersionSelection,
    ShowErrorLogsOnly,
    Quit,
}

impl Action {
    // iterator for enum https://stackoverflow.com/a/21376984
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 3] = [
            Action::OpenNamespaceSelection,
            Action::OpenVersionSelection,
            Action::Quit,
        ];
        ACTIONS.iter()
    }

    pub fn keys(&self) -> &[Key] {
        match self {
            Action::OpenNamespaceSelection => &[Key::Char('n')],
            Action::OpenVersionSelection => &[Key::Char('v')],
            Action::ShowErrorLogsOnly => &[Key::Ctrl('e')],
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
            Action::OpenNamespaceSelection => write!(f, "Select namespace"),
            Action::OpenVersionSelection => write!(f, "Select version"),
            Action::ShowErrorLogsOnly => write!(f, "Show error logs only"),
            Action::Quit => write!(f, "Quit"),
        }
    }
}
