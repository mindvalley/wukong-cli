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

#[derive(EnumIter, Debug)]
enum MetricTypeFilter {
    CpuUtilization,
    MemoryComponents,
    ConnectionsCount,
}

type MetricHash = HashMap<MetricType, MetricValue>;

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

#[derive(Debug, Clone)]
enum MetricValue {
    CpuUtilization(f64),
    MemoryComponents(MetricsMemoryComponents),
    ConnectionsCount(i64),
}

#[derive(Debug, Clone, Copy)]
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

    pub async fn get_log_entries(
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

    pub async fn get_access_token_info(&self) -> Result<TokenInfo, GCloudError> {
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
    pub async fn get_database_metrics(
        &self,
        project_id: &str,
    ) -> Result<Vec<DatabaseMetrics>, GCloudError> {
        let mut database_metrics = Vec::new();
        let current_time = Utc::now();
        let start_time = current_time - Duration::minutes(3);
        let mut responses: Vec<ListTimeSeriesResponse> = Vec::new();

        for metric_type in MetricTypeFilter::iter() {
            let bearer_token = format!("Bearer {}", self.access_token);
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

            let request: ListTimeSeriesRequest = generate_request(
                format!("{}", metric_type).as_str(),
                project_id,
                &start_time,
                &current_time,
            );

            let response = service.list_time_series(Request::new(request.clone()));

            responses.push(response.await?.into_inner());
        }
        let mut grouped_responses: HashMap<String, MetricHash> = HashMap::new();

        for response in &responses {
            if !response.time_series.is_empty() {
                let database_ids: Vec<Option<&String>> = response
                    .time_series
                    .iter()
                    .map(|time_series| {
                        time_series
                            .resource
                            .as_ref()
                            .unwrap()
                            .labels
                            .get("database_id")
                    })
                    .collect();

                for database_id in database_ids {
                    match database_id {
                        Some(database_id) => {
                            if grouped_responses.get(database_id).is_none() {
                                grouped_responses.insert(database_id.to_string(), HashMap::new());
                            };
                            let mut metrics_values = grouped_responses[database_id].clone();

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
                                        let cpu_utilization_point =
                                            response.time_series[0].points[0].clone();
                                        let cpu_utilization = match cpu_utilization_point.value {
                                            Some(typed_value) => match typed_value {
                                                TypedValue { value } => match value {
                                                    Some(google::monitoring::v3::typed_value::Value::DoubleValue(
                                                        double_value,
                                                    )) => double_value * 100.0, // Convert to percentage
                                                    _ => 0.0,
                                                },
                                            },
                                            None => 0.0,
                                        };
                                        metrics_values.insert(
                                            MetricType::CpuUtilization,
                                            MetricValue::CpuUtilization(cpu_utilization),
                                        );
                                    }
                                    MetricType::MemoryComponents => {
                                        let memory_cache_point =
                                            response.time_series[0].points[0].clone();
                                        let memory_cache = match memory_cache_point.value {
                                            Some(typed_value) => match typed_value {
                                                TypedValue { value } => match value {
                                                    Some(google::monitoring::v3::typed_value::Value::DoubleValue(
                                                        double_value,
                                                    )) => double_value, // Already a percentage
                                                    _ => 0.0,
                                                },
                                            },
                                            None => 0.0,
                                        };

                                        let memory_free_point =
                                            response.time_series[1].points[0].clone();
                                        let memory_free = match memory_free_point.value {
                                            Some(typed_value) => match typed_value {
                                                TypedValue { value } => match value {
                                                    Some(google::monitoring::v3::typed_value::Value::DoubleValue(
                                                        double_value,
                                                    )) => double_value, // Already a percentage
                                                    _ => 0.0,
                                                },
                                            },
                                            None => 0.0,
                                        };

                                        let memory_usage_point =
                                            response.time_series[2].points[0].clone();
                                        let memory_usage = match memory_usage_point.value {
                                            Some(typed_value) => match typed_value {
                                                TypedValue { value } => match value {
                                                    Some(google::monitoring::v3::typed_value::Value::DoubleValue(
                                                        double_value,
                                                    )) => double_value, // Already a percentage
                                                    _ => 0.0,
                                                },
                                            },
                                            None => 0.0,
                                        };

                                        metrics_values.insert(
                                            MetricType::MemoryComponents,
                                            MetricValue::MemoryComponents(
                                                MetricsMemoryComponents {
                                                    cache: memory_cache,
                                                    free: memory_free,
                                                    usage: memory_usage,
                                                },
                                            ),
                                        );
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
                                        metrics_values.insert(
                                            MetricType::ConnectionsCount,
                                            MetricValue::ConnectionsCount(2000),
                                        );

                                        // let connections_count_point =
                                        //     response.time_series[0].points[0].clone();
                                        // let connections_count= match connections_count_point.value {
                                        //     Some(typed_value) => match typed_value {
                                        //         TypedValue { value } => match value {
                                        //             Some(google::monitoring::v3::typed_value::Value::Int64Value(
                                        //                 int_value,
                                        //             )) => int_value,
                                        //             _ => 0,
                                        //         },
                                        //     },
                                        //     None => 0,
                                        // };
                                        // todo!("Connections Count response {:?}", response);
                                        // metrics_values.insert(
                                        //     MetricType::ConnectionsCount,
                                        //     MetricValue::ConnectionsCount(connections_count),
                                        // );
                                    }
                                },
                                Err(_err) => continue,
                            }

                            grouped_responses.insert(database_id.to_string(), metrics_values);
                        }
                        None => {
                            continue; // We don't do anything with the response if we can't get the database_id
                        }
                    }
                }
            }
        }

        grouped_responses.keys().into_iter().for_each(|key| {
            let cpu_utilization_option = grouped_responses
                .get(key)
                .unwrap()
                .get(&MetricType::CpuUtilization)
                .unwrap()
                .clone();

            let cpu_utilization = match cpu_utilization_option {
                MetricValue::CpuUtilization(cpu_utilization) => cpu_utilization,
                _ => 0.0,
            };

            let memory_components_option = grouped_responses
                .get(key)
                .unwrap()
                .get(&MetricType::MemoryComponents)
                .unwrap()
                .clone();

            let memory_usage = match memory_components_option {
                MetricValue::MemoryComponents(memory_components) => memory_components.usage,
                _ => 0.0,
            };

            let memory_free = match memory_components_option {
                MetricValue::MemoryComponents(memory_components) => memory_components.free,
                _ => 0.0,
            };

            let memory_cache = match memory_components_option {
                MetricValue::MemoryComponents(memory_components) => memory_components.cache,
                _ => 0.0,
            };

            let connections_counts_option = grouped_responses
                .get(key)
                .unwrap()
                .get(&MetricType::ConnectionsCount)
                .unwrap()
                .clone();

            let connections_count = match connections_counts_option {
                MetricValue::ConnectionsCount(connections_count) => connections_count,
                _ => 0,
            };

            database_metrics.push(DatabaseMetrics {
                name: key.to_string(),
                cpu_utilization,
                memory_usage: memory_usage.clone(),
                memory_free: memory_free.clone(),
                memory_cache: memory_cache.clone(),
                connections_count: connections_count.clone(),
            });
        });

        Ok(database_metrics)
    }
}

/// Functions from Google Cloud service.
impl WKClient {
    /// Get log entries from Google Cloud.
    pub async fn get_gcloud_log_entries(
        &self,
        options: LogEntriesOptions,
        access_token: String,
    ) -> Result<LogEntries, WKError> {
        let google_client = GCloudClient::new(access_token);
        google_client
            .get_log_entries(options)
            .await
            .map_err(|err| err.into())
    }

    // Get access token info from Google Cloud.
    pub async fn get_access_token_info(&self, access_token: String) -> Result<TokenInfo, WKError> {
        let google_client = GCloudClient::new(access_token);
        google_client
            .get_access_token_info()
            .await
            .map_err(|err| err.into())
    }

    pub async fn get_gcloud_database_metrics(
        &self,
        project_id: &str,
        access_token: String,
    ) -> Result<Vec<DatabaseMetrics>, WKError> {
        let google_client = GCloudClient::new(access_token);
        google_client
            .get_database_metrics(project_id)
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
