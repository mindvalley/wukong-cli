use std::time::Instant;

use ratatui::widgets::ScrollbarState;
use tokio::sync::mpsc::Sender;
use wukong_sdk::services::gcloud::google::logging::{r#type::LogSeverity, v2::LogEntry};

use crate::config::Config;

use super::{
    action::Action,
    events::{key::Key, network::NetworkEvent},
    ui::{
        namespace_selection::NamespaceSelectionWidget, version_selection::VersionSelectionWidget,
    },
    CurrentScreen, StatefulList,
};

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub const MAX_LOG_ENTRIES_LENGTH: usize = 1_000;

pub struct State {
    pub current_application: String,
    pub current_namespace: Option<String>,
    pub current_version: Option<String>,
    pub show_namespace_selection: bool,

    // loading state
    pub is_fetching_builds: bool,
    pub is_fetching_deployments: bool,
    pub is_checking_namespaces: bool,
    pub is_fetching_log_entries: bool,
    pub is_checking_version: bool,
    pub start_polling_log_entries: bool,

    // fetch data
    pub builds: Vec<Build>,
    pub deployments: Vec<Deployment>,
    pub has_log_errors: bool,
    pub log_entries: Vec<LogEntry>,
    pub log_entries_length: usize,

    pub last_log_entry_timestamp: Option<String>,
    // ui controls
    pub logs_vertical_scroll_state: ScrollbarState,
    pub logs_horizontal_scroll_state: ScrollbarState,
    pub logs_vertical_scroll: usize,
    pub logs_horizontal_scroll: usize,
    pub logs_enable_auto_scroll_to_bottom: bool,

    // For log entries polling
    pub instant_since_last_log_entries_poll: Instant,

    // ui state
    pub logs_widget_height: u16,
    pub logs_widget_width: u16,
    pub logs_tailing: bool,
    pub logs_severity: Option<LogSeverity>,
    pub show_search_bar: bool,
    pub search_bar_input: Input,
}

pub struct App {
    pub state: State,
    pub namespace_selections: StatefulList<String>,
    pub version_selections: StatefulList<String>,
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

        let mut version_selections =
            StatefulList::with_items(vec![String::from("green"), String::from("blue")]);
        version_selections.select(0);

        Self {
            state: State {
                current_application: config.core.application.clone(),
                current_namespace: None,
                current_version: None,

                show_namespace_selection: false,
                is_fetching_builds: false,
                is_fetching_deployments: false,
                is_checking_namespaces: false,
                is_checking_version: false,
                is_fetching_log_entries: false,
                start_polling_log_entries: false,
                logs_enable_auto_scroll_to_bottom: true,

                builds: vec![],
                deployments: vec![],
                last_log_entry_timestamp: None,
                has_log_errors: false,
                log_entries_length: 0,
                log_entries: Vec::with_capacity(1_000),

                logs_vertical_scroll_state: ScrollbarState::default(),
                logs_horizontal_scroll_state: ScrollbarState::default(),
                logs_vertical_scroll: 0,
                logs_horizontal_scroll: 0,
                instant_since_last_log_entries_poll: Instant::now(),

                logs_widget_width: 0,
                logs_widget_height: 0,
                logs_tailing: true,
                logs_severity: None,
                show_search_bar: false,
                search_bar_input: Input::default(),
            },
            namespace_selections,
            version_selections,
            current_screen: CurrentScreen::Main,
            actions: vec![
                Action::OpenNamespaceSelection,
                Action::OpenVersionSelection,
                Action::ToggleLogsTailing,
                Action::ShowErrorAndAbove,
                Action::Quit,
            ],
            network_event_sender: sender,
        }
    }

    pub async fn update(&mut self) -> AppReturn {
        // Poll every 10 seconds
        let poll_interval_ms = 10_000;
        let elapsed = self
            .state
            .instant_since_last_log_entries_poll
            .elapsed()
            .as_millis();

        if !self.state.start_polling_log_entries || elapsed >= poll_interval_ms {
            if !self.state.start_polling_log_entries {
                // only to show loader on the first call
                self.state.is_fetching_log_entries = true;

                // reset scroll state, it could be triggered when user switch namespace
                self.state.logs_vertical_scroll_state = ScrollbarState::default();
                self.state.logs_horizontal_scroll_state = ScrollbarState::default();
                self.state.logs_vertical_scroll = 0;
                self.state.logs_horizontal_scroll = 0;
            }

            self.state.start_polling_log_entries = true;
            self.state.instant_since_last_log_entries_poll = Instant::now();

            if self.state.logs_tailing {
                self.dispatch(NetworkEvent::GetGCloudLogs).await;
            }
        }

        AppReturn::Continue
    }

    pub async fn handle_input(&mut self, key: Key) -> AppReturn {
        match self.current_screen {
            CurrentScreen::NamespaceSelection => {
                NamespaceSelectionWidget::handle_input(key, self).await;
                AppReturn::Continue
            }
            CurrentScreen::VersionSelection => {
                VersionSelectionWidget::handle_input(key, self).await;
                AppReturn::Continue
            }
            CurrentScreen::LogSearchBar => {
                let cont_input = self.state.search_bar_input.handle_input(key);
                if !cont_input {
                    self.state.show_search_bar = false;
                    self.current_screen = CurrentScreen::Main;
                }

                AppReturn::Continue
            }
            _ => {
                match Action::from_key(key) {
                    Some(Action::OpenNamespaceSelection) => {
                        self.current_screen = CurrentScreen::NamespaceSelection;
                        AppReturn::Continue
                    }
                    Some(Action::OpenVersionSelection) => {
                        self.current_screen = CurrentScreen::VersionSelection;
                        AppReturn::Continue
                    }
            Some(Action::Quit) => AppReturn::Exit,
            Some(Action::ToggleLogsTailing) => {
                self.state.logs_tailing = !self.state.logs_tailing;
                AppReturn::Continue
            }
            Some(Action::ShowErrorAndAbove) => {
                self.dispatch(NetworkEvent::GetGCloudLogs).await;

                self.state.is_fetching_log_entries = true;
                self.state.start_polling_log_entries = false;

                self.state.log_entries = vec![];
                self.state.log_entries_length = 0;
                // Need to reset scroll, or else it will be out of bound

                // Add if not already in the list
                // or else remove it
                self.state.logs_severity = match self.state.logs_severity {
                    Some(LogSeverity::Error) => None,
                    _ => Some(LogSeverity::Error),
                };

                AppReturn::Continue
            }
            // TODO: just for prototype purpose
            // we will need to track current selected panel to apply the event
            None => match key {
                Key::Up | Key::Char('k') => {
                    self.state.logs_vertical_scroll =
                        self.state.logs_vertical_scroll.saturating_sub(5);
                    self.state.logs_vertical_scroll_state = self
                        .state
                        .logs_vertical_scroll_state
                        .position(self.state.logs_vertical_scroll);



                        self.state.is_fetching_log_entries = true;
                        self.state.start_polling_log_entries = false;

                        self.state.log_entries = vec![];
                        self.state.log_entries_length = 0;
                        // Need to reset scroll, or else it will be out of bound

                        // Add if not already in the list
                        // or else remove it
                        self.state.logs_serverity = match self.state.logs_serverity {
                            Some(LogSeverity::Error) => None,
                            _ => Some(LogSeverity::Error),
                        };

                        AppReturn::Continue
                    }
                    Some(Action::SearchLogs) => {
                        self.state.show_search_bar = !self.state.show_search_bar;

                        // focus on the log search bar so the search bar will handle the input
                        if self.state.show_search_bar {
                            self.current_screen = CurrentScreen::LogSearchBar;
                        } else {
                            self.current_screen = CurrentScreen::Main;
                        }

                        AppReturn::Continue
                    }
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

                            self.state.logs_enable_auto_scroll_to_bottom = false;

                            AppReturn::Continue
                        }
                        Key::Down | Key::Char('j') => {
                            self.state.logs_vertical_scroll =
                                self.state.logs_vertical_scroll.saturating_add(5);
                            self.state.logs_vertical_scroll_state = self
                                .state
                                .logs_vertical_scroll_state
                                .position(self.state.logs_vertical_scroll as u16);

                            self.state.logs_enable_auto_scroll_to_bottom = false;

                            AppReturn::Continue
                        }
                        Key::Left | Key::Char('h') => {
                            self.state.logs_horizontal_scroll =
                                self.state.logs_horizontal_scroll.saturating_sub(5);
                            self.state.logs_horizontal_scroll_state = self
                                .state
                                .logs_horizontal_scroll_state
                                .position(self.state.logs_horizontal_scroll as u16);

                            self.state.logs_enable_auto_scroll_to_bottom = false;

                            AppReturn::Continue
                        }
                        Key::Right | Key::Char('l') => {
                            self.state.logs_horizontal_scroll =
                                self.state.logs_horizontal_scroll.saturating_add(5);
                            self.state.logs_horizontal_scroll_state = self
                                .state
                                .logs_horizontal_scroll_state
                                .position(self.state.logs_horizontal_scroll as u16);

                            self.state.logs_enable_auto_scroll_to_bottom = false;

                            AppReturn::Continue
                        }
                        _ => AppReturn::Continue,
                    },
                }
            }
        }
    }

    pub async fn dispatch(&self, network_event: NetworkEvent) {
        if let Err(e) = self.network_event_sender.send(network_event).await {
            println!("Error from network event: {}", e)
        }
    }
}

pub struct Input {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub cursor_position: usize,
    /// Current input mode
    pub input_mode: InputMode,
}

pub enum InputMode {
    Normal,
    Editing,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            cursor_position: 0,
        }
    }
}

impl Input {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn handle_input(&mut self, key: Key) -> bool {
        match key {
            Key::Char(to_insert) => {
                self.enter_char(to_insert);
                true
            }
            Key::Backspace => {
                self.delete_char();
                true
            }
            Key::Left => {
                self.move_cursor_left();
                true
            }
            Key::Right => {
                self.move_cursor_right();
                true
            }
            Key::Esc => {
                // self.input_mode = InputMode::Normal;
                self.input = "".to_string();
                false
            }
            _ => true,
        }
        // match self.input_mode {
        //     InputMode::Normal => match key {
        //         Key::Char('e') => {
        //             self.input_mode = InputMode::Editing;
        //         }
        //         Key::Char('q') => {
        //             // return Ok(());
        //         }
        //         _ => {}
        //     },
        //     InputMode::Editing => match key {
        //         // Key::Enter => self.submit_message(),
        //         Key::Char(to_insert) => {
        //             self.enter_char(to_insert);
        //         }
        //         Key::Backspace => {
        //             self.delete_char();
        //         }
        //         Key::Left => {
        //             self.move_cursor_left();
        //         }
        //         Key::Right => {
        //             self.move_cursor_right();
        //         }
        //         Key::Esc => {
        //             self.input_mode = InputMode::Normal;
        //         }
        //         _ => {}
        //     },
        //     _ => {}
        // }
    }
}
