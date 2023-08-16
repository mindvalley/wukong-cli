use crate::config::Config;

use super::{action::Action, CurrentScreen, StatefulList};

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
}
