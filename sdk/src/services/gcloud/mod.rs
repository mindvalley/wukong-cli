#[rustfmt::skip]
#[path = "api"]
pub mod google {
    #[path = ""]
    pub mod logging {
        #[path = "google.logging.r#type.rs"]
        pub mod r#type;
        #[path = "google.logging.v2.rs"]
        pub mod v2;
    }
    #[path = ""]
    pub mod monitoring {
        #[path = "google.monitoring.v3.rs"]
        pub mod v3;
    }
    #[path = ""]
     pub mod cloud {
         #[path = "google.cloud.sql.v1.rs"]
         pub mod sql;
     }
    #[path = "google.api.rs"]
    pub mod api;
    #[path = "google.rpc.rs"]
    pub mod rpc;

}

use self::google::{
    logging::v2::{log_entry, LogEntry},
    monitoring::v3::{
        aggregation::{Aligner, Reducer},
        list_time_series_request::TimeSeriesView,
        metric_service_client::MetricServiceClient,
        Aggregation, ListTimeSeriesRequest, ListTimeSeriesResponse, TimeInterval, TypedValue,
    },
};
use crate::{
    error::{GCloudError, WKError},
    WKClient,
};
use chrono::{DateTime, Duration, Utc};
use google::logging::v2::{
    logging_service_v2_client::LoggingServiceV2Client, ListLogEntriesRequest,
};

use hyper::{header::HeaderValue, HeaderMap};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{collections::HashMap, fmt::Display};

use strum::{EnumIter, IntoEnumIterator};
use tonic::{metadata::MetadataValue, transport::Channel, Request};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum MetricType {
    CpuUtilization,
    MemoryComponents,
    ConnectionsCount,
}

#[derive(EnumIter, Debug, PartialEq, Eq)]
enum MetricTypeFilter {
    CpuUtilization,
    MemoryComponents,
    ConnectionsCount,
}

#[derive(Debug, Default, Clone)]
struct CloudSQLMetrics {
    instance: String,
    cpu_utilization: f64,
    memory_components: MetricsMemoryComponents,
    connections_count: i64,
    max_connections_count: i64,
}

#[derive(Debug, Deserialize)]
struct DbInstanceSetting {
    #[serde(rename = "databaseFlags")]
    database_flags: Vec<DbFlags>,
}

#[derive(Debug, Deserialize)]
struct DbFlags {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct DbInstance {
    name: String,
    settings: Option<DbInstanceSetting>,
}

#[derive(Debug, Deserialize)]
struct DbInstanceList {
    items: Vec<DbInstance>,
}

impl Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::CpuUtilization => {
                write!(f, "cloudsql.googleapis.com/database/cpu/utilization")
            }
            MetricType::MemoryComponents => {
                write!(f, "cloudsql.googleapis.com/database/memory/components")
            }
            MetricType::ConnectionsCount => {
                write!(
                    f,
                    "cloudsql.googleapis.com/database/postgresql/num_backends"
                )
            }
        }
    }
}

impl FromStr for MetricType {
    type Err = GCloudError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cloudsql.googleapis.com/database/cpu/utilization" => Ok(MetricType::CpuUtilization),
            "cloudsql.googleapis.com/database/memory/components" => {
                Ok(MetricType::MemoryComponents)
            }
            "cloudsql.googleapis.com/database/postgresql/num_backends" => {
                Ok(MetricType::ConnectionsCount)
            }
            _ => Err(GCloudError::InvalidMetricType),
        }
    }
}

impl Display for MetricTypeFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricTypeFilter::CpuUtilization => {
                write!(f, "metric.type=\"{}\"", MetricType::CpuUtilization)
            }
            MetricTypeFilter::MemoryComponents => {
                write!(f, "metric.type=\"{}\"", MetricType::MemoryComponents)
            }
            MetricTypeFilter::ConnectionsCount => {
                write!(f, "metric.type=\"{}\"", MetricType::ConnectionsCount)
            }
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct MetricsMemoryComponents {
    cache: f64,
    free: f64,
    usage: f64,
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "time={} ", self.timestamp.as_ref().unwrap())?;
        write!(f, "level={} ", self.severity().as_str_name())?;

        match self.payload.as_ref().unwrap() {
            log_entry::Payload::ProtoPayload(payload) => {
                write!(f, "proto_payload={:?}", payload)?;
            }
            log_entry::Payload::TextPayload(payload) => {
                write!(f, "text_payload={:?}", payload)?;
            }
            log_entry::Payload::JsonPayload(payload) => {
                let keys = payload.fields.keys().collect::<Vec<_>>();
                let value = keys
                    .iter()
                    .filter(|k| payload.fields.get(**k).is_some())
                    .map(|k| {
                        format!(
                            "{}: {}",
                            k,
                            display_prost_type_value_kind(
                                payload.fields.get(*k).unwrap().kind.clone()
                            )
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "json_payload={{ {value} }}")?;
            }
        };
        writeln!(f)?;
        Ok(())
    }
}

fn display_prost_type_value_kind(kind: Option<prost_types::value::Kind>) -> String {
    if let Some(kind) = kind {
        match kind {
            prost_types::value::Kind::NullValue(_) => "null".to_string(),
            prost_types::value::Kind::NumberValue(value) => {
                format!("{value}")
            }
            prost_types::value::Kind::StringValue(value) => format!("{:?}", value)
                .replace('\"', r#"""#)
                .replace("\\n", "")
                .replace('\\', "")
                .split(' ')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
            prost_types::value::Kind::BoolValue(value) => {
                format!("{value}")
            }
            prost_types::value::Kind::StructValue(value) => {
                let keys = value.fields.keys().collect::<Vec<_>>();
                let value = keys
                    .iter()
                    .filter(|k| value.fields.get(**k).is_some())
                    .map(|k| {
                        format!(
                            "{}: {}",
                            k,
                            display_prost_type_value_kind(
                                value.fields.get(*k).unwrap().kind.clone()
                            )
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{{ {value} }}")
            }
            prost_types::value::Kind::ListValue(value) => {
                let values = value
                    .values
                    .iter()
                    .map(|v| display_prost_type_value_kind(v.kind.clone()))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("[{values}]")
            }
        }
    } else {
        "null".to_string()
    }
}

#[derive(Debug, Default)]
pub struct LogEntriesOptions {
    pub project_ids: Option<Vec<String>>,
    pub filter: Option<String>,
    pub page_size: Option<i32>,
    pub page_token: Option<String>,
    pub order_by: Option<String>,
    pub resource_names: Option<Vec<String>>,
}

impl From<LogEntriesOptions> for ListLogEntriesRequest {
    fn from(value: LogEntriesOptions) -> Self {
        ListLogEntriesRequest {
            filter: value.filter.unwrap_or_default(),
            page_size: value.page_size.unwrap_or_default(),
            page_token: value.page_token.unwrap_or_default(),
            order_by: value.order_by.unwrap_or_default(),
            resource_names: value.resource_names.unwrap_or_default(),
        }
    }
}

pub struct LogEntriesTailOptions {
    pub filter: Option<String>,
    pub buffer_window: Option<Duration>,
    pub resource_names: Option<Vec<String>>,
}

pub struct LogEntries {
    pub entries: Option<Vec<google::logging::v2::LogEntry>>,
    pub next_page_token: Option<String>,
}

#[derive(Debug)]
pub struct DatabaseMetrics {
    pub name: String,
    pub cpu_utilization: f64,
    pub memory_usage: f64,
    pub memory_free: f64,
    pub memory_cache: f64,
    pub connections_count: i64,
    pub max_connections_count: i64,
}

pub struct GCloudClient {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub issued_to: String,
    pub audience: String,
    pub scope: String,
    pub expires_in: i64,
    pub access_type: String,
}

impl GCloudClient {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    pub async fn fetch_log_entries(
        &self,
        options: LogEntriesOptions,
    ) -> Result<LogEntries, GCloudError> {
        let bearer_token = format!("Bearer {}", self.access_token);
        let header_value: MetadataValue<_> = bearer_token.parse().unwrap();

        let channel = Channel::from_static("https://logging.googleapis.com")
            .connect()
            .await
            .unwrap();

        let mut service =
            LoggingServiceV2Client::with_interceptor(channel, move |mut req: Request<()>| {
                let metadata_map = req.metadata_mut();
                metadata_map.insert("authorization", header_value.clone());
                metadata_map.insert("user-agent", "grpc-go/1.14".parse().unwrap());

                Ok(req)
            });

        let request: ListLogEntriesRequest = options.into();

        let response = service
            .list_log_entries(Request::new(request))
            .await?
            .into_inner();

        Ok(LogEntries {
            entries: Some(response.entries),
            next_page_token: Some(response.next_page_token),
        })
    }

    pub async fn fetch_access_token_info(&self) -> Result<TokenInfo, GCloudError> {
        let token_info_url = "https://www.googleapis.com/oauth2/v1/tokeninfo";

        let query_params = vec![("access_token", self.access_token.clone())];

        let client = reqwest::Client::new();

        let response = client
            .get(token_info_url)
            .query(&query_params)
            .send()
            .await?;

        let token_info = match response.error_for_status() {
            Ok(token_info) => token_info.json::<TokenInfo>().await?,
            Err(err) => {
                return Err(GCloudError::ReqwestError(err));
            }
        };

        Ok(token_info)
    }

    /// Here we get the database metrics from Google Cloud by sending a request using the `MetricServiceClient`
    /// for each `MetricTypeFilter` that we have defined. The responses for the requests are then extracted into
    /// a Vector of `DatabaseMetrics`s that is updated on the App's `state.databases.database_metrics`.
    pub async fn fetch_database_metrics(
        &self,
        project_id: &str,
    ) -> Result<Vec<DatabaseMetrics>, GCloudError> {
        let mut database_metrics = Vec::new();
        let current_time = Utc::now();
        let start_time = current_time - Duration::try_minutes(3).unwrap();
        let mut responses: Vec<ListTimeSeriesResponse> = Vec::new();

        let bearer_token = format!("Bearer {}", self.access_token);

        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.append(
            reqwest::header::AUTHORIZATION,
            HeaderValue::from_str(&bearer_token).unwrap(),
        );

        let resp = client
            .get(format!(
                "https://sqladmin.googleapis.com/v1/projects/{}/instances",
                project_id
            ))
            .headers(headers)
            .send()
            .await?;

        let database_instances = resp.json::<DbInstanceList>().await?;

        let header_value: MetadataValue<_> = bearer_token.parse().unwrap();
        let channel = Channel::from_static("https://monitoring.googleapis.com")
            .connect()
            .await
            .unwrap();

        let mut service =
            MetricServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
                let metadata_map = req.metadata_mut();
                metadata_map.insert("authorization", header_value.clone());
                metadata_map.insert("user-agent", "grpc-go/1.14".parse().unwrap());

                Ok(req)
            });
        for metric_type in MetricTypeFilter::iter() {
            if metric_type == MetricTypeFilter::ConnectionsCount {
                let request: ListTimeSeriesRequest = generate_request_for_cloudsql(
                    format!("{}", metric_type).as_str(),
                    project_id,
                    &start_time,
                    &current_time,
                );

                let response = service
                    .list_time_series(Request::new(request.clone()))
                    .await?;

                responses.push(response.into_inner());
            } else {
                let request: ListTimeSeriesRequest = generate_request(
                    format!("{}", metric_type).as_str(),
                    project_id,
                    &start_time,
                    &current_time,
                );

                let response = service
                    .list_time_series(Request::new(request.clone()))
                    .await?;

                responses.push(response.into_inner());
            }
        }
        let mut grouped_responses: HashMap<String, CloudSQLMetrics> = HashMap::new();

        for response in &responses {
            if !response.time_series.is_empty() {
                let database_ids: Vec<&String> = response
                    .time_series
                    .iter()
                    .filter_map(|time_series| {
                        time_series
                            .resource
                            .as_ref()
                            .unwrap()
                            .labels
                            .get("database_id")
                    })
                    .collect();

                // deduplicate database_ids
                for id in database_ids {
                    if !grouped_responses.contains_key(id) {
                        grouped_responses.insert(id.to_string(), CloudSQLMetrics::default());
                    }
                }

                for (database_id, metrics_values) in grouped_responses.iter_mut() {
                    metrics_values.instance = database_id.to_string();

                    let metric_type = MetricType::from_str(
                        response.time_series[0]
                            .metric
                            .as_ref()
                            .unwrap()
                            .r#type
                            .as_str(),
                    );

                    match metric_type {
                        Ok(valid_type) => match valid_type {
                            MetricType::CpuUtilization => {
                                let points = response
                                    .time_series
                                    .iter()
                                    .filter(|time_series| {
                                        time_series
                                            .resource
                                            .as_ref()
                                            .unwrap()
                                            .labels
                                            .get("database_id")
                                            == Some(database_id)
                                    })
                                    .filter_map(|each| each.points.first())
                                    .collect::<Vec<_>>();

                                // Assuming there is only one in the time series data has the same database_id for the cpu metric
                                // so we can just take the first point
                                let cpu_utilization = if let Some(first_point) = points.first() {
                                    match first_point.value {
                                        Some(TypedValue { value: Some(google::monitoring::v3::typed_value::Value::DoubleValue(value)) }) => value * 100.0,
                                        _ => 0.0,
                                    }
                                } else {
                                    0.0
                                };

                                metrics_values.cpu_utilization = cpu_utilization;
                            }
                            // For the other MetricTypes, we've aggregated them so that there
                            // is only one TimeSeries with one point in the response, so we can
                            // just take the first. But for MemoryComponents, we get three TimeSeries,
                            // one for each component (cache, free, usage).
                            MetricType::MemoryComponents => {
                                let memory_cache = response
                                    .time_series
                                    .iter()
                                    .filter(|time_series| {
                                        time_series
                                            .resource
                                            .as_ref()
                                            .unwrap()
                                            .labels
                                            .get("database_id")
                                            == Some(database_id)
                                    })
                                    .find(|each| {
                                        each.metric.as_ref().unwrap().labels.get("component")
                                            == Some(&"Cache".to_string())
                                    }).map_or(0.0, |each| {
                                        match each.points.first() {
                                            Some(first_point) => match first_point.value {
                                                Some(TypedValue { value: Some(google::monitoring::v3::typed_value::Value::DoubleValue(value)) }) => value,
                                                _ => 0.0,
                                            },
                                            _ => 0.0,
                                        }
                                    });

                                let memory_free = response
                                    .time_series
                                    .iter()
                                    .filter(|time_series| {
                                        time_series
                                            .resource
                                            .as_ref()
                                            .unwrap()
                                            .labels
                                            .get("database_id")
                                            == Some(database_id)
                                    })
                                    .find(|each| {
                                        each.metric.as_ref().unwrap().labels.get("component")
                                            == Some(&"Free".to_string())
                                    }).map_or(0.0, |each| {
                                        match each.points.first() {
                                            Some(first_point) => match first_point.value {
                                                Some(TypedValue { value: Some(google::monitoring::v3::typed_value::Value::DoubleValue(value)) }) => value,
                                                _ => 0.0,
                                            },
                                            _ => 0.0,
                                        }
                                    });

                                let memory_usage = response
                                    .time_series
                                    .iter()
                                    .filter(|time_series| {
                                        time_series
                                            .resource
                                            .as_ref()
                                            .unwrap()
                                            .labels
                                            .get("database_id")
                                            == Some(database_id)
                                    })
                                    .find(|each| {
                                        each.metric.as_ref().unwrap().labels.get("component")
                                            == Some(&"Usage".to_string())
                                    }).map_or(0.0, |each| {
                                        match each.points.first() {
                                            Some(first_point) => match first_point.value {
                                                Some(TypedValue { value: Some(google::monitoring::v3::typed_value::Value::DoubleValue(value)) }) => value,
                                                _ => 0.0,
                                            },
                                            _ => 0.0,
                                        }
                                    });

                                metrics_values.memory_components = MetricsMemoryComponents {
                                    cache: memory_cache,
                                    free: memory_free,
                                    usage: memory_usage,
                                };
                            }
                            // TODO: Implement the connections count metric. Currently we are able
                            // get the metric from the response which returns a list of time series
                            // for connections by apps (e.g. `mv_wukong_api_proxy_db`, `cloudsqladmin`
                            // and `postgres`). However, we are unable to get an aggregated sum of the
                            // number of connections (which is what we want to display in the dashboard),
                            // i.e. the total number of connections across all apps.
                            //
                            // We tried adding a Reducer::ReduceSum to the aggregation for the request,
                            // but that returned an empty list. We tried a few combinations of Aggregator
                            // and Reducer without much luck.
                            //
                            // To complete this feature, we need to figure out to either get the sum of
                            // the connections across all apps, or to get the connections for only the
                            // app (e.g. `mv_wukong_api_proxy_db`), which adds the requirement of knowing
                            // the app name (which we don't have at the moment).
                            //
                            MetricType::ConnectionsCount => {
                                let max_connections_count = database_instances
                                    .items
                                    .iter()
                                    .find(|each| {
                                        each.name
                                            == database_id.split(':').collect::<Vec<&str>>()[1]
                                    })
                                    .map_or(0, |instance| match instance.settings {
                                        Some(ref settings) => settings
                                            .database_flags
                                            .iter()
                                            .find(|flag| flag.name == "max_connections")
                                            .map_or(0, |flag| {
                                                flag.value.parse::<i64>().unwrap_or(0)
                                            }),
                                        None => 0,
                                    });

                                let count = response
                                    .time_series
                                    .iter()
                                    .find(|time_series| {
                                        time_series
                                            .resource
                                            .as_ref()
                                            .unwrap()
                                            .labels
                                            .get("database_id")
                                            == Some(database_id)
                                    })
                                    .map_or(0.0, |time_series| {
                                            if let Some(first_point) = time_series.points.first() {
                                                if let Some(typed_value) = &first_point.value {
                                                    let TypedValue { value } = typed_value;
                                                    match value {
                                                        Some(google::monitoring::v3::typed_value::Value::DoubleValue(
                                                            value,
                                                        )) => *value,
                                                        _ => 0.0,
                                                    }
                                                } else {
                                                    0.0
                                                }
                                            } else {
                                                0.0
                                            }
                                    });

                                metrics_values.connections_count = count.ceil() as i64;
                                metrics_values.max_connections_count = max_connections_count;
                            }
                        },
                        Err(_err) => continue,
                    }
                }
            }
        }

        grouped_responses.iter().for_each(|(key, value)| {
            database_metrics.push(DatabaseMetrics {
                name: key.to_string(),
                cpu_utilization: value.cpu_utilization,
                memory_usage: value.memory_components.usage,
                memory_free: value.memory_components.free,
                memory_cache: value.memory_components.cache,
                connections_count: value.connections_count,
                max_connections_count: value.max_connections_count,
            });
        });

        Ok(database_metrics)
    }
}

/// Functions from Google Cloud service.
impl WKClient {
    /// Get log entries from Google Cloud.
    pub async fn fetch_gcloud_log_entries(
        &self,
        options: LogEntriesOptions,
        access_token: String,
    ) -> Result<LogEntries, WKError> {
        let google_client = GCloudClient::new(access_token);
        google_client
            .fetch_log_entries(options)
            .await
            .map_err(|err| err.into())
    }

    // Get access token info from Google Cloud.
    pub async fn fetch_access_token_info(
        &self,
        access_token: String,
    ) -> Result<TokenInfo, WKError> {
        let google_client = GCloudClient::new(access_token);
        google_client
            .fetch_access_token_info()
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_gcloud_database_metrics(
        &self,
        project_id: &str,
        access_token: String,
    ) -> Result<Vec<DatabaseMetrics>, WKError> {
        let google_client = GCloudClient::new(access_token);
        google_client
            .fetch_database_metrics(project_id)
            .await
            .map_err(|err| err.into())
    }
}

// For all working metric types, we are able to use this generic request with Aligner::AlignMean
// and Reducer::ReduceNone to get the metric values. However, for the connections count metric, we
// might need to introduce a different request to get the metric values, either by introducing a
// a new variable to this function, or introducing a separate function for it.
fn generate_request(
    metric_type: &str,
    project_id: &str,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> ListTimeSeriesRequest {
    ListTimeSeriesRequest {
        name: format!("projects/{}", project_id),
        filter: metric_type.to_string(),
        interval: Some(TimeInterval {
            start_time: Some(Timestamp {
                seconds: start_time.timestamp(),
                nanos: 0,
            }),
            end_time: Some(Timestamp {
                seconds: end_time.timestamp(),
                nanos: 0,
            }),
        }),
        view: TimeSeriesView::Full.into(),
        aggregation: Some(Aggregation {
            alignment_period: Some(prost_types::Duration {
                seconds: 180,
                nanos: 0,
            }),
            per_series_aligner: Aligner::AlignMean as i32,
            cross_series_reducer: Reducer::ReduceNone as i32,
            group_by_fields: Vec::new(),
        }),
        secondary_aggregation: None,
        order_by: "".to_string(),
        page_size: 10,
        page_token: "".to_string(),
    }
}

fn generate_request_for_cloudsql(
    metric_type: &str,
    project_id: &str,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> ListTimeSeriesRequest {
    ListTimeSeriesRequest {
        name: format!("projects/{}", project_id),
        filter: metric_type.to_string(),
        interval: Some(TimeInterval {
            start_time: Some(Timestamp {
                seconds: start_time.timestamp(),
                nanos: 0,
            }),
            end_time: Some(Timestamp {
                seconds: end_time.timestamp(),
                nanos: 0,
            }),
        }),
        view: TimeSeriesView::Full.into(),
        aggregation: Some(Aggregation {
            alignment_period: Some(prost_types::Duration {
                seconds: 180,
                nanos: 0,
            }),
            per_series_aligner: Aligner::AlignMean as i32,
            cross_series_reducer: Reducer::ReduceSum as i32,
            group_by_fields: vec!["resource.labels.database_id".to_string()],
        }),
        secondary_aggregation: None,
        order_by: "".to_string(),
        page_size: 10,
        page_token: "".to_string(),
    }
}
