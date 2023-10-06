use wukong_sdk::services::gcloud::google::logging::r#type::LogSeverity;

use crate::commands::tui::{
    action::Action,
    app::{ActiveBlock, App, AppReturn, DialogContext},
    events::{key::Key, network::NetworkEvent},
};

use super::common_key_events;

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::Log));
        }
        key if common_key_events::up_event(key) => {
            let new_scroll_position = app.state.logs_vertical_scroll.saturating_sub(5);
            handle_vertical_scroll(app, new_scroll_position)
        }
        key if common_key_events::down_event(key) => {
            let new_scroll_position = app.state.logs_vertical_scroll.saturating_add(5);
            handle_vertical_scroll(app, new_scroll_position)
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
                Some(ActiveBlock::Dialog(DialogContext::LogSearchBar)),
                Some(ActiveBlock::Dialog(DialogContext::LogSearchBar)),
            );

            if app.state.show_search_bar {
                app.state.show_filter_bar = false;
            }
        }
        key if Action::from_key(key) == Some(Action::FilterLogs) => {
            app.state.show_filter_bar = true;

            app.set_current_route_state(
                Some(ActiveBlock::Dialog(DialogContext::LogIncludeFilter)),
                Some(ActiveBlock::Dialog(DialogContext::LogIncludeFilter)),
            );

            if app.state.show_filter_bar {
                app.state.show_search_bar = false;
            }
        }
        _ => {}
    };

    AppReturn::Continue
}

async fn handle_show_error_and_above(app: &mut App) {
    app.dispatch(NetworkEvent::GetGCloudLogs).await;

    app.state.is_fetching_log_entries = true;
    app.state.start_polling_log_entries = false;

    app.state.log_entries = vec![];
    app.state.log_entries_length = 0;
    // Need to reset scroll, or else it will be out of bound

    // Add if not already in the list
    // or else remove it
    app.state.logs_severity = match app.state.logs_severity {
        Some(LogSeverity::Error) => None,
        _ => Some(LogSeverity::Error),
    };
}

fn handle_vertical_scroll(app: &mut App, new_scroll_position: usize) {
    app.state.logs_vertical_scroll = new_scroll_position;
    app.state.logs_vertical_scroll_state = app
        .state
        .logs_vertical_scroll_state
        .position(app.state.logs_vertical_scroll as u16);

    app.state.logs_enable_auto_scroll_to_bottom = false;
}

fn handle_horizontal_scroll(app: &mut App, new_scroll_position: usize) {
    app.state.logs_horizontal_scroll = new_scroll_position;
    app.state.logs_horizontal_scroll_state = app
        .state
        .logs_horizontal_scroll_state
        .position(app.state.logs_horizontal_scroll as u16);

    app.state.logs_enable_auto_scroll_to_bottom = false;
}
