use crate::commands::tui::{
    action::Action,
    app::{ActiveBlock, App, AppReturn, DialogContext},
    events::{key::Key, network::NetworkEvent},
};
use wukong_sdk::services::gcloud::google::logging::r#type::LogSeverity;

use super::common_key_events;

// In the absence of an selected block, handle standard events as usual.
pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match Action::from_key(key) {
        Some(Action::Quit) => AppReturn::Exit,
        Some(Action::OpenNamespaceSelection) => open_dialog(app, DialogContext::NamespaceSelection),
        Some(Action::OpenVersionSelection) => open_dialog(app, DialogContext::VersionSelection),
        Some(Action::SearchLogs) => {
            app.set_current_route_state(Some(ActiveBlock::Log), Some(ActiveBlock::Log));

            app.state.show_search_bar = !app.state.show_search_bar;
            if app.state.show_search_bar {
                app.state.show_filter_bar = false;
            }
            AppReturn::Continue
        }
        Some(Action::FilterLogs) => {
            app.set_current_route_state(Some(ActiveBlock::Log), Some(ActiveBlock::Log));

            app.state.show_filter_bar = !app.state.show_filter_bar;
            if app.state.show_filter_bar {
                app.state.show_search_bar = false;
            }
            AppReturn::Continue
        }
        Some(Action::ToggleLogsTailing) => {
            app.state.logs_tailing = !app.state.logs_tailing;
            AppReturn::Continue
        }
        Some(Action::ShowErrorAndAbove) => {
            show_error_and_above_logs(app).await;
            AppReturn::Continue
        }
        _ => handle_key_events(key, app),
    }
}

fn handle_key_events(key: Key, app: &mut App) -> AppReturn {
    match key {
        Key::Enter => {
            let current_hovered = app.get_current_route().hovered_block;
            app.set_current_route_state(Some(current_hovered), None);

            AppReturn::Continue
        }
        key if common_key_events::down_event(key) => match app.get_current_route().hovered_block {
            ActiveBlock::Log => {
                app.set_current_route_state(None, Some(ActiveBlock::Build));
                AppReturn::Continue
            }
            _ => AppReturn::Continue,
        },
        key if common_key_events::up_event(key) => match app.get_current_route().hovered_block {
            ActiveBlock::Build => {
                app.set_current_route_state(None, Some(ActiveBlock::Log));
                AppReturn::Continue
            }
            ActiveBlock::Deployment => {
                app.set_current_route_state(None, Some(ActiveBlock::Log));
                AppReturn::Continue
            }
            _ => AppReturn::Continue,
        },
        key if common_key_events::right_event(key) => match app.get_current_route().hovered_block {
            ActiveBlock::Build => {
                app.set_current_route_state(None, Some(ActiveBlock::Deployment));
                AppReturn::Continue
            }
            _ => AppReturn::Continue,
        },
        key if common_key_events::left_event(key) => match app.get_current_route().hovered_block {
            ActiveBlock::Deployment => {
                app.set_current_route_state(None, Some(ActiveBlock::Build));
                AppReturn::Continue
            }
            _ => AppReturn::Continue,
        },

        _ => AppReturn::Continue,
    }
}

fn open_dialog(app: &mut App, dialog_context: DialogContext) -> AppReturn {
    app.set_current_route_state(Some(ActiveBlock::Dialog(dialog_context)), None);
    AppReturn::Continue
}

async fn show_error_and_above_logs(app: &mut App) -> AppReturn {
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

    AppReturn::Continue
}
