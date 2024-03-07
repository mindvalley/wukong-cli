use super::{
    appsignal::reset_appsignal_panel_and_trigger_appsignal_refetch, common_key_events,
    databases::reset_database_panel_and_trigger_database_refetch,
    logs::reset_log_panel_and_trigger_log_refetch,
};

use crate::commands::tui::{
    app::{App, AppReturn, Block},
    events::{key::Key, network::NetworkEvent},
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.set_current_route_state(
                Some(Block::Empty),
                Some(Block::Middle(app.state.selected_tab)),
            );
        }
        key if common_key_events::back_event(key) => app.push_navigation_stack(Block::Empty),
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

    app.push_navigation_stack(Block::Empty)
}

async fn fetch_and_reset_polling(app: &mut App, selected_version: String) {
    app.state.current_namespace = Some(selected_version);
    reset_log_panel_and_trigger_log_refetch(app);

    // We also need to refresh the builds and appsignal data upon namespace change
    // as these data are based on the current namespace
    app.dispatch(NetworkEvent::GetBuilds).await;

    reset_appsignal_panel_and_trigger_appsignal_refetch(app);
    app.dispatch(NetworkEvent::GetAppsignalData).await;

    reset_database_panel_and_trigger_database_refetch(app);
    app.dispatch(NetworkEvent::GetDatabaseMetrics).await;
}
