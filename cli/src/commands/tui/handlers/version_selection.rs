use super::common_key_events;

use crate::commands::tui::{
    app::{ActiveBlock, App, AppReturn},
    events::{key::Key, network::NetworkEvent},
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::Log));
        }
        key if common_key_events::back_event(key) => app.push_navigation_stack(ActiveBlock::Empty),
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
            app.push_navigation_stack(ActiveBlock::Empty);
            return;
        }
    }

    fetch_and_reset_polling(app, selected_version).await;
    app.push_navigation_stack(ActiveBlock::Empty);
}

async fn fetch_and_reset_polling(app: &mut App, selected_version: String) {
    app.state.current_version = Some(selected_version);
    app.state.log_entries = vec![];
    app.state.log_entries_length = app.state.log_entries.len();

    app.state.is_fetching_log_entries = true;
    app.state.start_polling_log_entries = false;

    // reset error state
    app.state.log_entries_error = None;

    app.dispatch(NetworkEvent::GetBuilds).await;
}
