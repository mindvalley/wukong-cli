use crate::commands::tui::{
    app::{App, AppReturn, AppsignalState},
    events::key::Key,
};

pub async fn handler(_key: Key, _app: &mut App) -> AppReturn {
    AppReturn::Continue
}

pub fn reset_appsignal_panel_and_trigger_appsignal_refetch(app: &mut App) {
    app.state.appsignal = AppsignalState::default();
    app.state.is_fetching_appsignal_data = true;

    // this will make the appsignal api handler to get the config from the application config
    // since the namespace is changed
    app.state.is_appsignal_enabled = None;
    app.state.appsignal_app_id = None;
    app.state.appsignal_namespace = None;

    // this will trigger refetch in tui/app.rs update()
    app.state.start_polling_appsignal_data = false;

    app.state.appsignal_error = None;
}
