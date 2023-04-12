use crate::{
    commands::Context,
    error::CliError,
    loader::new_spinner_progress_bar,
    services::gcloud::{GCloudClient, LogEntriesOptions},
};

pub async fn handle_logs_demo(_context: Context) -> Result<bool, CliError> {
    let auth_progress_bar = new_spinner_progress_bar();
    auth_progress_bar.set_message("Checking authentication status...");

    let client = GCloudClient::new().await;

    auth_progress_bar.finish_and_clear();

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching log entries ... ");

    let log = client
        .get_log_entries(LogEntriesOptions {
            resource_names: Some(vec!["projects/mv-stg-applications-hub".to_string()]),
            page_size: Some(5),
            ..Default::default()
        })
        .await?;

    progress_bar.finish_and_clear();

    eprintln!("entries {:#?}", log.entries.unwrap_or(vec![]));
    eprintln!("next_page_token {:?}", log.next_page_token);

    Ok(true)
}
