use std::{fmt::Display, slice::Iter};

use super::events::key::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    OpenNamespaceSelection,
    OpenVersionSelection,
    ShowErrorAndAbove,
    ShowDatabaseStatus,
    ToggleLogsTailing,
    ShowLogs,
    SearchLogs,
    FilterLogs,
    // TimeFilterLogs,
    ExpandToFullScreen,
    LineWrapLogs,
    Refresh,
    Quit,
}

impl Action {
    // iterator for enum https://stackoverflow.com/a/21376984
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 12] = [
            Action::OpenNamespaceSelection,
            Action::OpenVersionSelection,
            Action::ToggleLogsTailing,
            Action::ShowErrorAndAbove,
            Action::ShowDatabaseStatus,
            Action::ShowLogs,
            Action::SearchLogs,
            Action::FilterLogs,
            // Action::TimeFilterLogs,
            Action::ExpandToFullScreen,
            Action::LineWrapLogs,
            Action::Refresh,
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
            Action::ShowDatabaseStatus => &[Key::Char('d')],
            Action::ShowLogs => &[Key::Char('g')],
            Action::SearchLogs => &[Key::Ctrl('s')],
            Action::FilterLogs => &[Key::Ctrl('f')],
            Action::ExpandToFullScreen => &[Key::Ctrl('w')],
            Action::LineWrapLogs => &[Key::Ctrl('l')],
            Action::Quit => &[Key::Char('q')],
            // Action::TimeFilterLogs => &[Key::Ctrl('y')],
            Action::Refresh => &[Key::Ctrl('r')],
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
            Action::ShowDatabaseStatus => write!(f, "Show database status"),
            Action::Quit => write!(f, "Quit"),
            Action::ShowLogs => write!(f, "Show logs"),
            Action::SearchLogs => write!(f, "Search logs"),
            Action::FilterLogs => write!(f, "Filter logs"),
            Action::LineWrapLogs => write!(f, "Line wrap logs"),
            // Action::TimeFilterLogs => write!(f, "Filter logs by time"),
            Action::ExpandToFullScreen => write!(f, "Expand to full screen"),
            Action::Refresh => write!(f, "Refresh"),
        }
    }
}
