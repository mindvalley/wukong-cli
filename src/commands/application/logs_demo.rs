use crate::{
    commands::Context,
    error::CliError,
    loader::new_spinner_progress_bar,
    services::gcloud::{GCloudClient, LogEntriesOptions, LogEntriesTailOptions},
};

pub async fn handle_logs_demo(_context: Context) -> Result<bool, CliError> {
    let gcloud_client = GCloudClient::new().await?;

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching log entries ... ");

    let log = gcloud_client
        .get_log_entries(LogEntriesOptions {
            project_ids: None,
            filter: None,
            page_size: Some(5),
            page_token: None,
            order_by: None,
            resource_names: Some(vec!["projects/mv-stg-applications-hub".to_string()]),
        })
        .await?;

    let _log_tail = gcloud_client
        .get_log_entries_tail(LogEntriesTailOptions {
            resource_names: Some(vec!["projects/mv-stg-applications-hub".to_string()]),
            filter: None,
            buffer_window: None,
        })
        .await?;

    progress_bar.finish_and_clear();

    eprintln!("entries {:#?}", log.entries.unwrap_or(vec![]));
    eprintln!("next_page_token {:?}", log.next_page_token);

    Ok(true)
}
