use std::{collections::HashMap, time::Instant};

use ratatui::{
    prelude::Rect,
    widgets::{ScrollbarState, TableState},
};
use tokio::sync::mpsc::Sender;
use wukong_sdk::services::gcloud::google::logging::{r#type::LogSeverity, v2::LogEntry};

use crate::config::Config;

use super::{action::Action, events::network::NetworkEvent, StatefulList};

const DEFAULT_ROUTE: Route = Route {
    active_block: Block::Empty,
    hovered_block: Block::Log,
};

#[derive(Default)]
pub struct Input {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub cursor_position: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DialogContext {
    NamespaceSelection,
    VersionSelection,
    LogSearch,
    LogIncludeFilter,
    LogExcludeFilter,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Block {
    Build,
    Deployment,
    Log,
    Empty,
    Dialog(DialogContext),
}

#[derive(Debug)]
pub struct Route {
    pub active_block: Block,
    pub hovered_block: Block,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct BlockInfo {
    pub block_id: Block,
    pub top_left_corner: Option<(u16, u16)>,
    pub bottom_right_corner: Option<(u16, u16)>,
}

pub const MAX_LOG_ENTRIES_LENGTH: usize = 2_000;

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
    pub logs_table_current_start_index: usize,
    // last index of the table that is visible
    pub logs_table_current_last_index: usize,
    // whether last index of the table that is visible is fully rendered
    // useful to know if we need to scroll during textwrap
    pub logs_table_current_last_fully_rendered: bool,
    pub expanded_block: Option<Block>,

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
    pub logs_textwrap: bool,

    pub logs_size: (u16, u16),
}

pub struct App {
    pub state: State,
    pub namespace_selections: StatefulList<String>,
    pub version_selections: StatefulList<String>,
    pub actions: Vec<Action>,
    pub network_event_sender: Sender<NetworkEvent>,

    pub block_map: HashMap<Block, BlockInfo>,
    navigation_stack: Vec<Route>,
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

        let mut logs_table_state = TableState::default();
        logs_table_state.select(Some(0));

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
                logs_table_current_start_index: 0,
                logs_table_current_last_index: 0,
                logs_table_current_last_fully_rendered: true,
                expanded_block: None,

                logs_widget_width: 0,
                logs_widget_height: 0,
                logs_tailing: true,
                logs_severity: None,
                show_search_bar: false,
                show_filter_bar: false,
                search_bar_input: Input::default(),
                filter_bar_include_input: Input::default(),
                filter_bar_exclude_input: Input::default(),
                logs_textwrap: false,
                logs_size: (0, 0),
            },
            navigation_stack: vec![DEFAULT_ROUTE],
            block_map: HashMap::new(),
            namespace_selections,
            version_selections,
            actions: vec![
                Action::OpenNamespaceSelection,
                Action::OpenVersionSelection,
                Action::ToggleLogsTailing,
                Action::ShowErrorAndAbove,
                Action::Quit,
                Action::SearchLogs,
                Action::FilterLogs,
                Action::ExpandToFullScreen,
                Action::LineWrapLogs,
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

    pub async fn dispatch(&self, network_event: NetworkEvent) {
        if let Err(e) = self.network_event_sender.send(network_event).await {
            println!("Error from network event: {}", e)
        }
    }

    pub fn push_navigation_stack(&mut self, next_active_block: Block) {
        if !self
            .navigation_stack
            .last()
            .map(|last_route| last_route.active_block == next_active_block)
            .unwrap_or(false)
        {
            self.navigation_stack.push(Route {
                active_block: next_active_block,
                hovered_block: next_active_block,
            });
        }
    }

    fn get_current_route_mut(&mut self) -> &mut Route {
        self.navigation_stack.last_mut().unwrap()
    }

    pub fn get_current_route(&self) -> &Route {
        self.navigation_stack.last().unwrap_or(&DEFAULT_ROUTE)
    }

    pub fn set_current_route_state(
        &mut self,
        active_block: Option<Block>,
        hovered_block: Option<Block>,
    ) {
        let current_route = self.get_current_route_mut();

        if let Some(active_block) = active_block {
            current_route.active_block = active_block;
        }

        if let Some(hovered_block) = hovered_block {
            current_route.hovered_block = hovered_block;
        }
    }

    pub fn update_draw_lock(&mut self, current_block: Block, rect: Rect) {
        if let Some(block) = self.block_map.get_mut(&current_block) {
            block.top_left_corner = Some((rect.x, rect.y));
            block.bottom_right_corner = Some((rect.x + rect.width, rect.y + rect.height));
        } else {
            self.block_map.insert(
                current_block,
                BlockInfo {
                    block_id: current_block,
                    top_left_corner: Some((rect.x, rect.y)),
                    bottom_right_corner: Some((rect.x + rect.width, rect.y + rect.height)),
                },
            );
        }
    }
}
