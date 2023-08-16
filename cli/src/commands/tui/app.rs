use crate::config::Config;

use super::{
    action::Action, events::key::Key, ui::namespace_selection::NamespaceSelectionWidget,
    CurrentScreen, StatefulList,
};

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub struct State {
    pub current_application: String,
    pub current_namespace: String,
    pub show_namespace_selection: bool,
}

pub struct App {
    pub state: State,
    pub namespace_selections: StatefulList<String>,
    pub current_screen: CurrentScreen,
    pub actions: Vec<Action>,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let mut namespace_selections =
            StatefulList::with_items(vec![String::from("prod"), String::from("staging")]);
        namespace_selections.select(0);

        Self {
            state: State {
                current_application: config.core.application.clone(),
                current_namespace: String::from("prod"),
                show_namespace_selection: false,
            },
            namespace_selections,
            current_screen: CurrentScreen::Main,
            actions: vec![Action::SelectNamespace, Action::Quit],
        }
    }

    pub fn update(&mut self) -> AppReturn {
        AppReturn::Continue
    }

    pub fn handle_input(&mut self, key: Key) -> AppReturn {
        if let CurrentScreen::NamespaceSelection = self.current_screen {
            NamespaceSelectionWidget::handle_input(key, self);
            return AppReturn::Continue;
        }

        match Action::from_key(key) {
            Some(Action::SelectNamespace) => {
                self.current_screen = CurrentScreen::NamespaceSelection;
                AppReturn::Continue
            }
            Some(Action::Quit) => AppReturn::Exit,
            None => AppReturn::Continue,
        }
    }
}
