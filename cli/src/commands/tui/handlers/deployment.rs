use super::common_key_events;
use crate::commands::tui::{
    action::Action,
    app::{App, AppReturn, Block},
    events::key::Key,
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            if let Some(Block::Deployment) = app.state.expanded_block {
                app.state.expanded_block = None;
            } else {
                app.set_current_route_state(Some(Block::Empty), Some(Block::Deployment));
            }
        }
        key if Action::from_key(key) == Some(Action::ExpandToFullScreen) => {
            app.state.expanded_block = Some(Block::Deployment);
        }
        _ => {}
    };

    AppReturn::Continue
}
