use crate::commands::tui::{
    app::{App, AppReturn},
    events::key::Key,
};

pub async fn handler(_key: Key, _app: &mut App) -> AppReturn {
    AppReturn::Continue
}
