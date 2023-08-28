use super::{ApplicationNamespace, ApplicationVersion};
use crate::{
    auth, commands::Context, config::Config, error::WKCliError, loader::new_spinner,
    wukong_client::WKClient,
};
use aion::*;
use chrono::{DateTime, Local};
use log::{debug, trace};
use once_cell::sync::Lazy;
use openidconnect::url;
use owo_colors::OwoColorize;
use regex::Regex;
use wukong_sdk::services::gcloud::LogEntriesOptions;
use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[wukong_telemetry(command_event = "application_logs")]
#[allow(clippy::too_many_arguments)]
pub async fn handle_logs(
    context: Context,
    namespace: &ApplicationNamespace,
    version: &ApplicationVersion,
    show_error_and_above: &bool,
    since: &Option<String>,
    until: &Option<String>,
    limit: &i32,
    include: &Vec<String>,
    exclude: &Vec<String>,
    url_mode: &bool,
) -> Result<bool, WKCliError> {
    let auth_loader = new_spinner();
    auth_loader.set_message("Checking if you're authenticated to Google Cloud...");

    let config = Config::load_from_default_path()?;
    let gcloud_access_token = auth::google_cloud::get_token_or_login().await;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    auth_loader.finish_and_clear();

    let application_loader = new_spinner();
    application_loader.set_message("Fetching application details ... ");

    let application_resp = wk_client
        .fetch_application_with_k8s_cluster(
            &context.current_application,
            &namespace.to_string(),
            &version.to_string(),
        )
        .await?
        .application;

    if let Some(application_data) = application_resp {
        if let Some(cluster) = application_data.k8s_cluster {
            let filter = generate_filter(
                version,
                &cluster.cluster_name,
                &cluster.k8s_namespace,
                since,
                until,
                show_error_and_above,
            )?;
            let resource_names = vec![format!("projects/{}", cluster.google_project_id)];
            application_loader.finish_and_clear();

            trace!("filter: {}", filter);
            trace!("resource_names: {:?}", resource_names);

            // url mode only return the url
            if *url_mode {
                let url = url::Url::parse(&format!(
                    "https://console.cloud.google.com/logs/query;query={}",
                    filter
                ))
                .unwrap();
                eprintln!(
                    "Copy and paste the 🔗 below to your browser:\n{}?project={}",
                    url, cluster.google_project_id
                );
                return Ok(true);
            }

            let fetch_loader = new_spinner();
            fetch_loader.set_message("Fetching log entries ... ");

            let log = wk_client
                .get_gcloud_log_entries(
                    LogEntriesOptions {
                        resource_names: Some(resource_names),
                        page_size: Some(*limit),
                        filter: Some(filter),
                        ..Default::default()
                    },
                    gcloud_access_token,
                )
                .await?;

            fetch_loader.finish_and_clear();

            // do include and exclude filtering
            let mut log_entries = log.entries.unwrap_or_default();

            if include.is_empty() && exclude.is_empty() {
                for entry in log_entries {
                    eprintln!("{}", entry);
                }
                return Ok(true);
            }

            if !exclude.is_empty() {
                let regexes = exclude
                    .iter()
                    .map(|each| Regex::new(&format!(r"(?i){}", each.trim())).unwrap())
                    .collect::<Vec<_>>();

                log_entries = log_entries
                    .into_iter()
                    .filter(|each| {
                        for regex in &regexes {
                            if regex.is_match(&each.to_string()) {
                                return false;
                            }
                        }
                        true
                    })
                    .collect::<Vec<_>>();
            }

            if !include.is_empty() {
                let regexes = include
                    .iter()
                    .map(|each| Regex::new(&format!(r"(?i){}", each.trim())).unwrap())
                    .collect::<Vec<_>>();

                log_entries = log_entries
                    .into_iter()
                    .filter(|each| {
                        for regex in &regexes {
                            if regex.is_match(&each.to_string()) {
                                return true;
                            }
                        }
                        false
                    })
                    .collect::<Vec<_>>();

                for each in log_entries {
                    let mut output_string = each.to_string();

                    let mut matches: Vec<(usize, usize)> = Vec::new();
                    for regex in &regexes {
                        for found in regex.find_iter(&output_string.clone()) {
                            let start = found.start();
                            let end = found.end();

                            // merge the match if it overlaps with any existing match
                            // to avoid highlighting issue
                            let mut is_matched = false;
                            for m in &mut matches {
                                if m.0 <= start && m.1 >= end {
                                    is_matched = true;
                                    break;
                                }

                                if m.0 < start && start < m.1 && end > m.1 {
                                    m.1 = end;
                                    is_matched = true;
                                    break;
                                }
                                if m.1 > end && end > m.0 && start < m.0 {
                                    m.0 = start;
                                    is_matched = true;
                                    break;
                                }
                            }

                            if !is_matched {
                                matches.push((start, end));
                            }
                        }
                    }

                    // sort the matches so the output will be correct
                    // since we are adding offset manually
                    matches.sort_by(|a, b| a.0.cmp(&b.0));

                    for (index, m) in matches.iter().enumerate() {
                        let offset = index * 10; // each color will add 10 bytes

                        output_string.replace_range(
                            (m.0 + offset)..(m.1 + offset),
                            &format!(
                                "{}",
                                output_string[(m.0 + offset)..(m.1 + offset)]
                                    .to_string()
                                    .cyan()
                            ),
                        );
                    }

                    eprintln!("{output_string}");
                }

                return Ok(true);
            }

            for entry in log_entries {
                eprintln!("{entry}");
            }
        }
    } else {
        eprintln!("The log is empty.");
    }

    Ok(true)
}

static TIMESTAMP_DAY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+d$").unwrap());
static TIMESTAMP_HOUR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+h$").unwrap());
static TIMESTAMP_MINUTE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+m$").unwrap());

fn generate_filter(
    version: &ApplicationVersion,
    cluster_name: &str,
    namespace_name: &str,
    since: &Option<String>,
    until: &Option<String>,
    show_error_and_above: &bool,
) -> Result<String, WKCliError> {
    let mut filter = String::new();
    filter.push_str(format!("resource.type=\"k8s_container\" AND resource.labels.cluster_name=\"{}\" AND resource.labels.namespace_name=\"{}\"", cluster_name, namespace_name).as_str());

    filter.push_str(" AND ");

    if let Some(since) = since {
        filter.push_str(&format!("timestamp>=\"{}\"", get_timestamp(since)?));
    } else {
        let one_hour_ago = (Local::now() - 1.hours()).to_rfc3339();
        filter.push_str(&format!("timestamp>=\"{one_hour_ago}\""));
    }

    if let Some(until) = until {
        if !filter.is_empty() {
            filter.push_str(" AND ");
        }

        filter.push_str(&format!("timestamp<=\"{}\"", get_timestamp(until)?));
    }

    if *show_error_and_above {
        if !filter.is_empty() {
            filter.push_str(" AND ");
        }

        filter.push_str("severity>=ERROR");
    }

    filter.push_str(" AND ");
    filter.push_str(&format!("resource.labels.pod_name:{}", version.to_string()));

    Ok(filter)
}

fn get_timestamp(timestamp: &String) -> Result<String, WKCliError> {
    match DateTime::parse_from_rfc3339(timestamp) {
        Ok(_) => Ok(timestamp.clone()),
        Err(e) => {
            if TIMESTAMP_DAY_REGEX.is_match(timestamp) {
                let now = Local::now();
                let num = timestamp.replace('d', "").parse::<i64>()?;
                Ok((now - num.days()).to_rfc3339())
            } else if TIMESTAMP_HOUR_REGEX.is_match(timestamp) {
                let now = Local::now();
                let num = timestamp.replace('h', "").parse::<i64>()?;
                Ok((now - num.hours()).to_rfc3339())
            } else if TIMESTAMP_MINUTE_REGEX.is_match(timestamp) {
                let now = Local::now();
                let num = timestamp.replace('m', "").parse::<i64>()?;
                Ok((now - num.minutes()).to_rfc3339())
            } else {
                debug!("Error parsing timestamp: {}", &timestamp);
                debug!("Error message: {:?}", e);
                Err(WKCliError::ChronoParseError {
                    value: timestamp.clone(),
                    source: e,
                })
            }
        }
    }
}
