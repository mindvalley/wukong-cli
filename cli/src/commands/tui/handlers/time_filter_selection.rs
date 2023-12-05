// use super::{common_key_events, logs::reset_log_panel_and_trigger_log_refetch};

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
            reset_log_panel_and_trigger_log_refetch(app);
        }
    } else {
        app.state.current_time_filter = Some(*selected);
        reset_log_panel_and_trigger_log_refetch(app);
    }

    app.push_navigation_stack(Block::Log)
}
