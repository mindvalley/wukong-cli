use std::time::Instant;

use ratatui::widgets::{ScrollbarState, TableState};
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
    pub log_entries: Vec<LogEntry>,
    pub log_entries_length: usize,
    pub log_entries_error: Option<String>,
    pub builds_error: Option<String>,
    pub deployments_error: Option<String>,

    pub last_log_entry_timestamp: Option<String>,
    // ui controls
    pub logs_vertical_scroll_state: ScrollbarState,
    pub logs_horizontal_scroll_state: ScrollbarState,
    pub logs_vertical_scroll: usize,
    pub logs_horizontal_scroll: usize,
    pub logs_enable_auto_scroll_to_bottom: bool,
    pub logs_table_state: TableState,
    pub logs_table_current_index: usize,
    pub logs_table_start_position: usize,

    // For log entries polling
    pub instant_since_last_log_entries_poll: Instant,

    // ui state
    pub logs_widget_height: u16,
    pub logs_widget_width: u16,
    pub logs_tailing: bool,
    pub logs_severity: Option<LogSeverity>,
    pub show_search_bar: bool,
    pub show_filter_bar: bool,
    pub search_bar_input: Input,
    pub filter_bar_include_input: Input,
    pub filter_bar_exclude_input: Input,
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
                is_fetching_builds: true,
                is_fetching_deployments: true,
                is_checking_namespaces: false,
                is_checking_version: false,
                is_fetching_log_entries: true,
                start_polling_log_entries: false,
                logs_enable_auto_scroll_to_bottom: true,

                builds: vec![],
                deployments: vec![],
                last_log_entry_timestamp: None,

                log_entries_length: 0,
                log_entries: Vec::with_capacity(1_000),
                log_entries_error: None,
                builds_error: None,
                deployments_error: None,

                logs_vertical_scroll_state: ScrollbarState::default(),
                logs_horizontal_scroll_state: ScrollbarState::default(),
                logs_vertical_scroll: 0,
                logs_horizontal_scroll: 0,
                instant_since_last_log_entries_poll: Instant::now(),
                logs_table_state: TableState::default(),
                logs_table_current_index: 0,
                logs_table_start_position: 0,

                logs_widget_width: 0,
                logs_widget_height: 0,
                logs_tailing: true,
                logs_severity: None,
                show_search_bar: false,
                show_filter_bar: false,
                search_bar_input: Input::default(),
                filter_bar_include_input: Input::default(),
                filter_bar_exclude_input: Input::default(),
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
                Action::SearchLogs,
                Action::FilterLogs,
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

        if self.state.current_namespace.is_some() && self.state.current_version.is_some() {
            // if this is the first log entries api call, fetch the log entries
            // even if the tail is not enabled
            if !self.state.start_polling_log_entries {
                // only to show loader on the first call
                self.state.is_fetching_log_entries = true;

                // reset scroll state, it could be triggered when user switch namespace
                self.state.logs_vertical_scroll_state = ScrollbarState::default();
                self.state.logs_horizontal_scroll_state = ScrollbarState::default();
                self.state.logs_vertical_scroll = 0;
                self.state.logs_horizontal_scroll = 0;

                self.state.start_polling_log_entries = true;
                self.state.instant_since_last_log_entries_poll = Instant::now();

                self.dispatch(NetworkEvent::GetGCloudLogs).await;
                return AppReturn::Continue;
            }

            // if this is not the first call, check if it's time to fetch more log entries
            // if yes, fetch the log entries if the tailing is enabled
            if elapsed >= poll_interval_ms {
                self.state.instant_since_last_log_entries_poll = Instant::now();

                if self.state.logs_tailing {
                    self.dispatch(NetworkEvent::GetGCloudLogs).await;
                }
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
                if cont_input {
                    // the log will stop tailing during search
                    self.state.logs_tailing = false;
                } else {
                    self.state.show_search_bar = false;
                    self.current_screen = CurrentScreen::Main;

                    // the log will resume tailing if the search is ended
                    self.state.logs_tailing = true;
                }

                AppReturn::Continue
            }
            CurrentScreen::LogFilterIncludeBar => {
                match key {
                    Key::Right => {
                        self.current_screen = CurrentScreen::LogFilterExcludeBar;
                    }
                    _ => {
                        let cont_input = self.state.filter_bar_include_input.handle_input(key);
                        if cont_input {
                            // the log will stop tailing during filtering
                            self.state.logs_tailing = false;
                        } else {
                            self.state.show_filter_bar = false;
                            self.current_screen = CurrentScreen::Main;

                            // the log will resume tailing if the filtering is ended
                            self.state.logs_tailing = true;
                        }
                    }
                }

                AppReturn::Continue
            }
            CurrentScreen::LogFilterExcludeBar => {
                match key {
                    Key::Left => {
                        self.current_screen = CurrentScreen::LogFilterIncludeBar;
                    }
                    _ => {
                        let cont_input = self.state.filter_bar_exclude_input.handle_input(key);
                        if cont_input {
                            // the log will stop tailing during filtering
                            self.state.logs_tailing = false;
                        } else {
                            self.state.show_filter_bar = false;
                            self.current_screen = CurrentScreen::Main;

                            // the log will resume tailing if the filtering is ended
                            self.state.logs_tailing = true;
                        }
                    }
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
                    Some(Action::SearchLogs) => {
                        self.state.show_search_bar = !self.state.show_search_bar;

                        // focus on the log search bar so the search bar will handle the input
                        if self.state.show_search_bar {
                            self.state.show_filter_bar = false;
                            self.current_screen = CurrentScreen::LogSearchBar;
                        } else {
                            self.current_screen = CurrentScreen::Main;
                        }

                        AppReturn::Continue
                    }
                    Some(Action::FilterLogs) => {
                        self.state.show_filter_bar = !self.state.show_filter_bar;

                        // focus on the log filter bar so the filter bar will handle the input
                        if self.state.show_filter_bar {
                            self.state.show_search_bar = false;
                            self.current_screen = CurrentScreen::LogFilterIncludeBar;
                        } else {
                            self.current_screen = CurrentScreen::Main;
                        }

                        AppReturn::Continue
                    }
                    // TODO: just for prototype purpose
                    // we will need to track current selected panel to apply the event
                    None => match key {
                        Key::Up | Key::Char('k') => {
                            // self.state.logs_vertical_scroll =
                            //     self.state.logs_vertical_scroll.saturating_sub(5);
                            // self.state.logs_vertical_scroll_state = self
                            //     .state
                            //     .logs_vertical_scroll_state
                            //     .position(self.state.logs_vertical_scroll as u16);
                            //
                            // self.state.logs_enable_auto_scroll_to_bottom = false;

                            let start_index = self.state.logs_table_start_position;
                            let num_rows = 12;

                            let mut i = self.state.logs_table_current_index;
                            i = if i == 0 { 0 } else { i - 1 };
                            self.state.logs_table_current_index = i;

                            if i >= start_index + num_rows {
                                self.state.logs_table_state.select(Some(11));
                            } else if i <= start_index {
                                self.state.logs_table_state.select(Some(0));
                            } else {
                                let selected = self.state.logs_table_state.selected().unwrap();
                                self.state.logs_table_state.select(Some(selected - 1));
                            }

                            self.state.logs_table_start_position = {
                                let current_index = self.state.logs_table_current_index;
                                if current_index <= start_index {
                                    // If it's past the first element, then show from that element downwards
                                    current_index
                                } else if current_index >= start_index + num_rows {
                                    current_index - num_rows + 1
                                } else {
                                    start_index
                                }
                            };

                            AppReturn::Continue
                        }
                        Key::Down | Key::Char('j') => {
                            // self.state.logs_vertical_scroll =
                            //     self.state.logs_vertical_scroll.saturating_add(5);
                            // self.state.logs_vertical_scroll_state = self
                            //     .state
                            //     .logs_vertical_scroll_state
                            //     .position(self.state.logs_vertical_scroll as u16);
                            //
                            // self.state.logs_enable_auto_scroll_to_bottom = false;

                            let mut i = self.state.logs_table_current_index;
                            i = if i >= self.state.log_entries.len() - 1 {
                                self.state.log_entries.len() - 1
                            } else {
                                i + 1
                            };
                            self.state.logs_table_current_index = i;
                            if i >= 12 {
                                if let Some(selected) = self.state.logs_table_state.selected() {
                                    if selected != 11 {
                                        self.state.logs_table_state.select(Some(selected - 1));
                                    }
                                }
                                self.state.logs_table_state.select(Some(11));
                            } else if i < 12 {
                                self.state.logs_table_state.select(Some(i));
                            }

                            let start_index = self.state.logs_table_start_position;
                            let num_rows = 12;
                            self.state.logs_table_start_position = {
                                let current_index = self.state.logs_table_current_index;
                                if current_index < start_index + num_rows {
                                    // If, using the current scroll position, we can see the element
                                    // (so within that and + num_rows) just reuse the current previously
                                    // scrolled position.
                                    start_index
                                } else if current_index >= start_index + num_rows {
                                    // If the current position past the last element visible in the list,
                                    // then skip until we can see that element.
                                    current_index - num_rows + 1
                                } else {
                                    0
                                }
                            };

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

#[derive(Default)]
pub struct Input {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub cursor_position: usize,
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
        // self.input.insert(self.cursor_position, new_char);
        self.input.push(new_char);

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
                self.input = "".to_string();
                self.reset_cursor();
                false
            }
            _ => true,
        }
    }
}
