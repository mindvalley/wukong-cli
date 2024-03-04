use crate::commands::tui::{
    app::{App, AppReturn, DatabasesState},
    events::key::Key,
};

pub async fn handler(_key: Key, app: &mut App) -> AppReturn {
    AppReturn::Continue
}

pub fn reset_database_panel_and_trigger_database_refetch(app: &mut App) {
    app.state.databases = DatabasesState::default();
    app.state.databases.is_fetching = true;

    app.state.databases.error = None;
}
