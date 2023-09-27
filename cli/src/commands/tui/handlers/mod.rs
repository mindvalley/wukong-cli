mod common_key_events;
mod empty;
mod logs;
mod namespace_selection;
mod version_selection;

use super::{
    app::{ActiveBlock, App, AppReturn, DialogContext},
    events::key::Key,
};

pub async fn input_handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.set_current_route_state(Some(ActiveBlock::Empty), None);
            AppReturn::Continue
        }
        key if common_key_events::exit_event(key) => AppReturn::Exit,
        _ => handle_block_events(key, app).await,
    }
}

async fn handle_block_events(key: Key, app: &mut App) -> AppReturn {
    let current_route = app.get_current_route();

    match current_route.active_block {
        ActiveBlock::Empty => empty::handler(key, app).await, // Main Input
        ActiveBlock::Log => logs::handler(key, app).await,
        ActiveBlock::Dialog(DialogContext::NamespaceSelection) => {
            namespace_selection::handler(key, app).await
        }
        ActiveBlock::Dialog(DialogContext::VersionSelection) => {
            version_selection::handler(key, app).await
        }
        _ => AppReturn::Continue,
    }
}
