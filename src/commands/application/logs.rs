use std::fmt::Display;

use super::{ApplicationNamespace, ApplicationVersion};
use crate::{
    commands::Context,
    error::CliError,
    loader::new_spinner_progress_bar,
    services::gcloud::{GCloudClient, LogEntries, LogEntriesOptions},
};
use aion::*;
use chrono::{DateTime, Local};
use log::debug;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

impl Display for LogEntries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(entries) = &self.entries {
            for entry in entries {
                // time="2023-03-03T06:41:28Z" level=info msg="getRepoObjs stats" application=mv-stg-spinnaker-appconsole build_options_ms=0 helm_ms=0 plugins_ms=0 repo_ms=0 time_ms=195 unmarshal_ms=195 version_ms=0
                write!(f, "time={} ", entry.timestamp.as_ref().unwrap())?;
                write!(f, "level={} ", entry.severity)?;
                write!(f, "msg={} ", entry.log_name)?;
                match entry.payload.as_ref().unwrap() {
                    crate::services::gcloud::google::logging::v2::log_entry::Payload::ProtoPayload(payload) => {
                        write!(f, "payload={:?}", payload)?;
                    },
                    crate::services::gcloud::google::logging::v2::log_entry::Payload::TextPayload(payload) => {
                        write!(f, "payload={:?}", payload)?;
                    },
                    crate::services::gcloud::google::logging::v2::log_entry::Payload::JsonPayload(payload) => {
                     let v: Value = serde_json::from_reader(payload).unwrap();
                        write!(f, "payload={:?}", payload.fields)?;
                    },
                };
                writeln!(f)?;
                // write!(f, "payload={} ", entry.payload.as_ref().unwrap_or_default().)?;
            }
        }

        Ok(())
    }
}

pub async fn handle_logs(
    _context: Context,
    _namespace: &ApplicationNamespace,
    _version: &ApplicationVersion,
    show_error_and_above: &bool,
    since: &Option<String>,
    until: &Option<String>,
    limit: &i32,
) -> Result<bool, CliError> {
    let auth_progress_bar = new_spinner_progress_bar();
    auth_progress_bar.set_message("Checking if you're authenticated to Google Cloud...");

    let gcloud_client = GCloudClient::new().await;

    auth_progress_bar.finish_and_clear();

    let filter = generate_filter(since, until, show_error_and_above)?;
    let resource_names = get_resource_names_from_api().await?;

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching log entries ... ");

    let log = gcloud_client
        .get_log_entries(LogEntriesOptions {
            resource_names: Some(resource_names),
            page_size: Some(*limit),
            filter: Some(filter),
            ..Default::default()
        })
        .await?;

    progress_bar.finish_and_clear();

    eprintln!("{log}");
    // eprintln!("next_page_token {:?}", log.next_page_token);

    Ok(true)
}

async fn get_resource_names_from_api() -> Result<Vec<String>, CliError> {
    Ok(vec!["projects/mv-stg-applications-hub".to_string()])
}

static TIMESTAMP_DAY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+d$").unwrap());
static TIMESTAMP_HOUR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+h$").unwrap());
static TIMESTAMP_MINUTE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+m$").unwrap());

fn generate_filter(
    since: &Option<String>,
    until: &Option<String>,
    show_error_and_above: &bool,
) -> Result<String, CliError> {
    let mut filter = String::new();
    filter.push_str("resource.type=\"k8s_container\" AND resource.labels.cluster_name=\"mv-stg-apphub-use4-gke-01\" AND resource.labels.namespace_name=\"mv-platform\"");

    filter.push_str(" AND ");

    if let Some(since) = since {
        filter.push_str(&format!("timestamp>=\"{}\"", get_timestamp(&since)?));
    } else {
        let one_hour_ago = (Local::now() - 1.hours()).to_rfc3339();
        filter.push_str(&format!("timestamp>=\"{one_hour_ago}\""));
    }

    if let Some(until) = until {
        if !filter.is_empty() {
            filter.push_str(" AND ");
        }

        filter.push_str(&format!("timestamp<=\"{}\"", get_timestamp(&until)?));
    }

    if *show_error_and_above {
        if !filter.is_empty() {
            filter.push_str(" AND ");
        }

        filter.push_str("severity>=ERROR");
    }

    Ok(filter)
}

fn get_timestamp(timestamp: &String) -> Result<String, CliError> {
    match DateTime::parse_from_rfc3339(&timestamp) {
        Ok(_) => Ok(timestamp.clone()),
        Err(e) => {
            if TIMESTAMP_DAY_REGEX.is_match(&timestamp) {
                let now = Local::now();
                let num = timestamp.replace("d", "").parse::<i64>()?;
                Ok((now - num.days()).to_rfc3339())
            } else if TIMESTAMP_HOUR_REGEX.is_match(&timestamp) {
                let now = Local::now();
                let num = timestamp.replace("h", "").parse::<i64>()?;
                Ok((now - num.hours()).to_rfc3339())
            } else if TIMESTAMP_MINUTE_REGEX.is_match(&timestamp) {
                let now = Local::now();
                let num = timestamp.replace("m", "").parse::<i64>()?;
                Ok((now - num.minutes()).to_rfc3339())
            } else {
                debug!("Error parsing timestamp: {}", &timestamp);
                debug!("Error message: {:?}", e);
                Err(CliError::ChronoParseError {
                    value: timestamp.clone(),
                    source: e,
                })
            }
        }
    }
}
