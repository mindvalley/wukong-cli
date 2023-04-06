use crate::{
    commands::Context,
    error::CliError,
    services::gcloud::{GCloudClient, LogEntriesOption},
};

pub async fn handle_logs_demo(_context: Context) -> Result<bool, CliError> {
    let log = GCloudClient::get_log_entries(LogEntriesOption {
        project_ids: None,
        filter: None,
        page_size: None,
        page_token: None,
        order_by: None,
        resource_names: Some(vec!["projects/mv-stg-applications-hub".to_string()]),
    })
    .await?;

    println!("entries {:?}", log.entries.unwrap_or(vec![]).len());
    println!("next_page_token {:?}", log.next_page_token);

    Ok(true)
}
