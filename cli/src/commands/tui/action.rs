use std::{fmt::Display, slice::Iter};

use super::events::key::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    OpenNamespaceSelection,
    OpenVersionSelection,
    ShowErrorAndAbove,
    ToggleLogsTailing,
    SearchLogs,
    FilterLogs,
    TimeFilterLogs,
    ExpandToFullScreen,
    Quit,
}

impl Action {
    // iterator for enum https://stackoverflow.com/a/21376984
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 9] = [
            Action::OpenNamespaceSelection,
            Action::OpenVersionSelection,
            Action::ToggleLogsTailing,
            Action::ShowErrorAndAbove,
            Action::SearchLogs,
            Action::FilterLogs,
            Action::TimeFilterLogs,
            Action::ExpandToFullScreen,
            Action::Quit,
        ];
        ACTIONS.iter()
    }

    pub fn keys(&self) -> &[Key] {
        match self {
            Action::OpenNamespaceSelection => &[Key::Char('n')],
            Action::OpenVersionSelection => &[Key::Char('v')],
            Action::ToggleLogsTailing => &[Key::Ctrl('t')],
            Action::ShowErrorAndAbove => &[Key::Ctrl('e')],
            Action::SearchLogs => &[Key::Ctrl('s')],
            Action::FilterLogs => &[Key::Ctrl('f')],
            Action::ExpandToFullScreen => &[Key::Ctrl('w')],
            Action::Quit => &[Key::Char('q')],
            Action::TimeFilterLogs => &[Key::Ctrl('r')],
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
            Action::ToggleLogsTailing => write!(f, "Toggle logs tailing"),
            Action::ShowErrorAndAbove => write!(f, "Show errors logs only"),
            Action::Quit => write!(f, "Quit"),
            Action::SearchLogs => write!(f, "Search logs"),
            Action::FilterLogs => write!(f, "Filter logs"),
            Action::TimeFilterLogs => write!(f, "Filter logs by time"),
            Action::ExpandToFullScreen => write!(f, "Expand to full screen"),
        }
    }
}
