use ratatui::widgets::ScrollbarState;
use tokio::sync::mpsc::Sender;
use wukong_sdk::services::gcloud::google::logging::v2::LogEntry;

use crate::config::Config;

use super::{
    action::Action,
    events::{key::Key, network::NetworkEvent},
    ui::namespace_selection::NamespaceSelectionWidget,
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

    // loading state
    pub is_fetching_builds: bool,
    pub is_fetching_deployments: bool,
    pub is_checking_namespaces: bool,
    pub is_fetching_logs: bool,

    // fetch data
    pub builds: Vec<Build>,
    pub deployments: Vec<Deployment>,
    pub log_entries: Vec<LogEntry>,
    // ui controls
    pub logs_vertical_scroll_state: ScrollbarState,
    pub logs_horizontal_scroll_state: ScrollbarState,
    pub logs_vertical_scroll: usize,
    pub logs_horizontal_scroll: usize,
}

pub struct App {
    pub state: State,
    pub namespace_selections: StatefulList<String>,
    pub current_screen: CurrentScreen,
    pub actions: Vec<Action>,
    pub network_event_sender: Sender<NetworkEvent>,
}

pub struct Build {
    pub name: String,
    pub commits: Vec<Commit>,
}
pub struct Commit {
    pub id: String,
    pub message_headline: String,
}

pub struct Deployment {
    pub name: String,
    pub environment: String,
    pub version: String,
    pub enabled: bool,
    pub deployed_ref: Option<String>,
    pub build_artifact: Option<String>,
    pub deployed_by: Option<String>,
    pub last_deployed_at: Option<i64>,
    pub status: Option<String>,
}

impl App {
    pub fn new(config: &Config, sender: Sender<NetworkEvent>) -> Self {
        let mut namespace_selections =
            StatefulList::with_items(vec![String::from("prod"), String::from("staging")]);
        namespace_selections.select(0);

        Self {
            state: State {
                current_application: config.core.application.clone(),
                current_namespace: String::from("prod"),
                show_namespace_selection: false,
                is_fetching_builds: true,
                is_fetching_deployments: true,
                is_checking_namespaces: true,
                is_fetching_logs: true,
                builds: vec![],
                deployments: vec![],
                log_entries: vec![],

                logs_vertical_scroll_state: ScrollbarState::default(),
                logs_horizontal_scroll_state: ScrollbarState::default(),
                logs_vertical_scroll: 0,
                logs_horizontal_scroll: 0,
            },
            namespace_selections,
            current_screen: CurrentScreen::Main,
            actions: vec![Action::OpenNamespaceSelection, Action::Quit],
            network_event_sender: sender,
        }
    }

    pub fn update(&mut self) -> AppReturn {
        AppReturn::Continue
    }

    pub async fn handle_input(&mut self, key: Key) -> AppReturn {
        if let CurrentScreen::NamespaceSelection = self.current_screen {
            NamespaceSelectionWidget::handle_input(key, self).await;
            return AppReturn::Continue;
        }

        match Action::from_key(key) {
            Some(Action::OpenNamespaceSelection) => {
                self.current_screen = CurrentScreen::NamespaceSelection;
                AppReturn::Continue
            }
            Some(Action::Quit) => AppReturn::Exit,
            // TODO: just for prototype purpose
            // we will need to track current selected panel to apply the event
            None => match key {
                Key::Up | Key::Char('k') => {
                    self.state.logs_vertical_scroll =
                        self.state.logs_vertical_scroll.saturating_sub(5);
                    self.state.logs_vertical_scroll_state = self
                        .state
                        .logs_vertical_scroll_state
                        .position(self.state.logs_vertical_scroll as u16);

                    AppReturn::Continue
                }
                Key::Down | Key::Char('j') => {
                    self.state.logs_vertical_scroll =
                        self.state.logs_vertical_scroll.saturating_add(5);
                    self.state.logs_vertical_scroll_state = self
                        .state
                        .logs_vertical_scroll_state
                        .position(self.state.logs_vertical_scroll as u16);

                    AppReturn::Continue
                }
                Key::Left | Key::Char('h') => {
                    self.state.logs_horizontal_scroll =
                        self.state.logs_horizontal_scroll.saturating_sub(5);
                    self.state.logs_horizontal_scroll_state = self
                        .state
                        .logs_horizontal_scroll_state
                        .position(self.state.logs_horizontal_scroll as u16);

                    AppReturn::Continue
                }
                Key::Right | Key::Char('l') => {
                    self.state.logs_horizontal_scroll =
                        self.state.logs_horizontal_scroll.saturating_add(5);
                    self.state.logs_horizontal_scroll_state = self
                        .state
                        .logs_horizontal_scroll_state
                        .position(self.state.logs_horizontal_scroll as u16);

                    AppReturn::Continue
                }
                _ => AppReturn::Continue,
            },
        }
    }

    pub async fn dispatch(&mut self, network_event: NetworkEvent) {
        if let Err(e) = self.network_event_sender.send(network_event).await {
            println!("Error from network event: {}", e)
        }
    }
}
