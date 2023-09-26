use wukong_sdk::services::gcloud::google::logging::r#type::LogSeverity;

use crate::commands::tui::{
    action::Action,
    app::{ActiveBlock, App, AppReturn, DialogContext},
    events::{key::Key, network::NetworkEvent},
};

// In the absence of an selected block, handle standard events as usual.
pub async fn handle_input(key: Key, app: &mut App) -> AppReturn {
    match Action::from_key(key) {
        Some(Action::Quit) => AppReturn::Exit,
        Some(Action::OpenNamespaceSelection) => {
            app.set_current_route_state(
                Some(ActiveBlock::Dialog(DialogContext::NamespaceSelection)),
                None,
            );
            AppReturn::Continue
        }
        Some(Action::OpenVersionSelection) => {
            app.set_current_route_state(
                Some(ActiveBlock::Dialog(DialogContext::VersionSelection)),
                None,
            );
            AppReturn::Continue
        }
        Some(Action::ToggleLogsTailing) => {
            app.state.logs_tailing = !app.state.logs_tailing;
            AppReturn::Continue
        }
        Some(Action::ShowErrorAndAbove) => {
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
        _ => match key {
            Key::Enter => {
                let current_hovered = app.get_current_route().hovered_block;
                app.set_current_route_state(Some(current_hovered), None);

                AppReturn::Continue
            }
            Key::Down | Key::Char('j') => match app.get_current_route().hovered_block {
                ActiveBlock::Log => {
                    app.set_current_route_state(None, Some(ActiveBlock::Build));
                    AppReturn::Continue
                }
                _ => AppReturn::Continue,
            },
            Key::Up | Key::Char('k') => match app.get_current_route().hovered_block {
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
            Key::Right | Key::Char('l') => match app.get_current_route().hovered_block {
                ActiveBlock::Build => {
                    app.set_current_route_state(None, Some(ActiveBlock::Deployment));
                    AppReturn::Continue
                }
                _ => AppReturn::Continue,
            },
            Key::Left | Key::Char('h') => match app.get_current_route().hovered_block {
                ActiveBlock::Deployment => {
                    app.set_current_route_state(None, Some(ActiveBlock::Build));
                    AppReturn::Continue
                }
                _ => AppReturn::Continue,
            },

            _ => AppReturn::Continue,
        },
    }
}
