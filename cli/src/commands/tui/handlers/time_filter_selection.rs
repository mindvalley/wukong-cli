use super::common_key_events;

use crate::commands::tui::{
    app::{App, AppReturn, Block},
    events::key::Key,
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.set_current_route_state(Some(Block::Log), Some(Block::Log));
        }
        key if common_key_events::back_event(key) => app.push_navigation_stack(Block::Empty),
        key if common_key_events::down_event(key) => app.time_filter_selections.next(),
        key if common_key_events::up_event(key) => app.time_filter_selections.previous(),
        Key::Enter => handle_enter_key(app).await,
        _ => {}
    };

    AppReturn::Continue
}

async fn handle_enter_key(app: &mut App) {
    let selected = app
        .time_filter_selections
        .items
        .get(app.time_filter_selections.state.selected().unwrap())
        .unwrap();

    if let Some(current_time_filter) = &app.state.current_time_filter {
        if current_time_filter != selected {
            app.state.current_time_filter = Some(*selected);
            fetch_and_reset_polling(app).await;
        }
    } else {
        app.state.current_time_filter = Some(*selected);
        fetch_and_reset_polling(app).await;
    }

    app.push_navigation_stack(Block::Log)
}

async fn fetch_and_reset_polling(app: &mut App) {
    app.state.log_entries = vec![];
    app.state.log_entries_length = app.state.log_entries.len();
    app.state.last_log_entry_timestamp = None;

    // this will trigger refetch of log entries
    app.state.is_fetching_log_entries = true;
    app.state.start_polling_log_entries = false;

    // reset error state
    app.state.log_entries_error = None;
}
