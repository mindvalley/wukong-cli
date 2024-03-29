use super::{common_key_events, log_filter_exclude, log_filter_include, log_search};
use crate::commands::tui::{
    action::Action,
    app::{App, AppReturn, Block, DialogContext, MAX_LOG_ENTRIES_LENGTH},
    events::key::Key,
};
use chrono::Utc;
use wukong_sdk::services::gcloud::google::logging::r#type::LogSeverity;

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            if let Some(Block::Middle(_)) = app.state.expanded_block {
                app.state.expanded_block = None;
            } else {
                app.set_current_route_state(
                    Some(Block::Empty),
                    Some(Block::Middle(app.state.selected_tab)),
                );
            }
        }
        key if common_key_events::up_event(key) => {
            // FIXME: currently we are scrolling up based on the current rendered count.
            // if 1 log rendered on the screen currently, then we will scroll up 1 log when up key
            // is pressed,
            // if 5 logs rendered on the screen currently, then we will scroll up 5 logs when up
            // key is pressed.
            // While this is working, but the UX is not great.
            // I haven't come up a better solution yet. We need a better calculation for this
            if app.state.logs_textwrap {
                let count = app.state.logs_table_current_last_index
                    - app.state.logs_table_current_start_index
                    + 1;

                app.state.logs_table_current_start_index = app
                    .state
                    .logs_table_current_start_index
                    .saturating_sub(count);
            } else {
                app.state.logs_table_current_start_index = app
                    .state
                    .logs_table_current_start_index
                    .saturating_sub(app.state.logs_size.1 as usize);
            }
        }
        key if common_key_events::down_event(key) => {
            let next_start_index = if app.state.logs_textwrap {
                if app.state.logs_table_current_last_fully_rendered {
                    app.state.logs_table_current_last_index.saturating_add(1)
                } else {
                    app.state.logs_table_current_last_index
                }
            } else {
                app.state
                    .logs_table_current_start_index
                    .saturating_add(app.state.logs_size.1 as usize)
            };

            // prevent going out of bounds
            if next_start_index >= app.state.log_entries.1.len() {
                app.state.logs_table_current_start_index =
                    app.state.log_entries.1.len().saturating_sub(1);
            } else {
                app.state.logs_table_current_start_index = next_start_index;
            }
        }
        key if common_key_events::left_event(key) => {
            let new_scroll_position = app.state.logs_horizontal_scroll.saturating_sub(5);
            handle_horizontal_scroll(app, new_scroll_position)
        }
        key if common_key_events::right_event(key) => {
            let new_scroll_position = app.state.logs_horizontal_scroll.saturating_add(5);
            handle_horizontal_scroll(app, new_scroll_position)
        }
        key if Action::from_key(key) == Some(Action::ToggleLogsTailing) => {
            app.state.logs_tailing = !app.state.logs_tailing;
        }
        key if Action::from_key(key) == Some(Action::ShowErrorAndAbove) => {
            handle_show_error_and_above(app).await;
        }
        key if Action::from_key(key) == Some(Action::SearchLogs) => {
            app.state.show_search_bar = true;

            app.set_current_route_state(
                Some(Block::Dialog(DialogContext::LogSearch)),
                Some(Block::Dialog(DialogContext::LogSearch)),
            );

            if app.state.show_search_bar {
                app.state.show_filter_bar = false;
                log_filter_exclude::reset_cursor(&mut app.state.filter_bar_exclude_input);
                log_filter_include::reset_cursor(&mut app.state.filter_bar_include_input);
            }
        }
        key if Action::from_key(key) == Some(Action::FilterLogs) => {
            app.state.show_filter_bar = true;

            app.set_current_route_state(
                Some(Block::Dialog(DialogContext::LogIncludeFilter)),
                Some(Block::Dialog(DialogContext::LogIncludeFilter)),
            );

            if app.state.show_filter_bar {
                app.state.show_search_bar = false;
                log_search::reset_cursor(&mut app.state.search_bar_input);
            }
        }
        key if Action::from_key(key) == Some(Action::ExpandToFullScreen) => {
            app.state.expanded_block = Some(Block::Middle(app.state.selected_tab));
        }
        key if Action::from_key(key) == Some(Action::LineWrapLogs) => {
            app.state.logs_textwrap = !app.state.logs_textwrap;

            // reset horizontal scroll position
            handle_horizontal_scroll(app, 0);
        }
        // key if Action::from_key(key) == Some(Action::TimeFilterLogs) => {
        //     app.set_current_route_state(Some(Block::Dialog(DialogContext::LogTimeFilter)), None);
        // }
        _ => {}
    };

    AppReturn::Continue
}

async fn handle_show_error_and_above(app: &mut App) {
    // Add if not already in the list
    // or else remove it
    app.state.logs_severity = match app.state.logs_severity {
        Some(LogSeverity::Error) => None,
        _ => Some(LogSeverity::Error),
    };

    reset_log_panel_and_trigger_log_refetch(app);
}

fn handle_horizontal_scroll(app: &mut App, new_scroll_position: usize) {
    app.state.logs_horizontal_scroll = new_scroll_position;
    app.state.logs_horizontal_scroll_state = app
        .state
        .logs_horizontal_scroll_state
        .position(app.state.logs_horizontal_scroll as u16);

    app.state.logs_enable_auto_scroll_to_bottom = false;
}

pub fn reset_log_panel_and_trigger_log_refetch(app: &mut App) {
    let new_id = Utc::now().timestamp();

    app.state.log_entries = (
        format!("{}", new_id),
        Vec::with_capacity(MAX_LOG_ENTRIES_LENGTH),
    );
    app.state.log_entries_length = 0;
    app.state.logs_table_current_start_index = 0;
    app.state.logs_table_current_last_index = 0;
    app.state.last_log_entry_timestamp = None;

    app.state.is_fetching_log_entries = true;
    // this will trigger refetch in tui/app.rs update()
    app.state.start_polling_log_entries = false;

    app.state.log_entries_error = None;
}
