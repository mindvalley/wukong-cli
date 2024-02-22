use crate::commands::tui::{
    action::Action, app::{App, AppReturn, Block}, events::key::Key
};

use super::common_key_events;

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            if let Some(Block::Database) = app.state.expanded_block {
                app.state.expanded_block = None;
            } else {
                app.set_current_route_state(Some(Block::Empty), Some(Block::Database));
            };
        }
        key if Action::from_key(key) == Some(Action::ExpandToFullScreen) => {
            app.state.expanded_block = Some(Block::Database);
        } 
        _ => {}
    }
    AppReturn::Continue
}
