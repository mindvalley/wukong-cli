use super::common_key_events;

use crate::commands::tui::{
    app::{ActiveBlock, App, AppReturn},
    events::{key::Key, network::NetworkEvent},
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => app.push_navigation_stack(ActiveBlock::Empty),
        key if common_key_events::down_event(key) => app.namespace_selections.next(),
        key if common_key_events::up_event(key) => app.namespace_selections.previous(),
        Key::Enter => handle_enter_key(app).await,
        _ => {}
    };

    AppReturn::Continue
}

async fn handle_enter_key(app: &mut App) {
    let selected = app
        .namespace_selections
        .items
        .get(app.namespace_selections.state.selected().unwrap())
        .unwrap();

    if let Some(current_namespace) = &app.state.current_namespace {
        if current_namespace != selected {
            fetch_and_reset_polling(app, selected.to_string()).await;
        }
    } else {
        fetch_and_reset_polling(app, selected.to_string()).await;
    }

    app.push_navigation_stack(ActiveBlock::Empty)
}

async fn fetch_and_reset_polling(app: &mut App, selected_version: String) {
    app.state.current_namespace = Some(selected_version);
    app.dispatch(NetworkEvent::GetBuilds).await;
    app.dispatch(NetworkEvent::GetGCloudLogs).await;

    app.state.is_fetching_log_entries = true;
    app.state.start_polling_log_entries = false;
    app.state.has_log_errors = false;
}
