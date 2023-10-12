use super::common_key_events;
use crate::commands::tui::{
    app::{ActiveBlock, App, AppReturn},
    events::key::Key,
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::Build));
        }
        _ => {}
    };

    AppReturn::Continue
}
