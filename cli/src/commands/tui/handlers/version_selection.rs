use super::{common_key_events, logs::reset_log_panel_and_trigger_log_refetch};

use crate::commands::tui::{
    app::{App, AppReturn, Block},
    events::{key::Key, network::NetworkEvent},
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.set_current_route_state(Some(Block::Empty), Some(Block::Log));
        }
        key if common_key_events::back_event(key) => app.push_navigation_stack(Block::Empty),
        key if common_key_events::down_event(key) => app.version_selections.next(),
        key if common_key_events::up_event(key) => app.version_selections.previous(),
        Key::Enter => handle_enter_key(app).await,
        _ => {}
    };

    AppReturn::Continue
}

async fn handle_enter_key(app: &mut App) {
    let selected_version_index = match app.version_selections.state.selected() {
        Some(index) => index,
        None => return, // No selected version, nothing to do
    };

    let selected_version = app.version_selections.items[selected_version_index].clone();

    if let Some(current_namespace) = &app.state.current_namespace {
        if selected_version == *current_namespace {
            app.push_navigation_stack(Block::Empty);
            return;
        }
    }

    fetch_and_reset_polling(app, selected_version).await;
    app.push_navigation_stack(Block::Empty);
}

async fn fetch_and_reset_polling(app: &mut App, selected_version: String) {
    app.state.current_version = Some(selected_version);
    reset_log_panel_and_trigger_log_refetch(app);

    app.dispatch(NetworkEvent::GetBuilds).await;
}
