#[rustfmt::skip]
#[path = "api"]
use std::fmt;
pub mod google {
    #[path = ""]
    pub mod logging {
        #[path = "google.logging.r#type.rs"]
        pub mod r#type;
        #[path = "google.logging.v2.rs"]
        pub mod v2;
    }
    // #[path = "google.api.rs"]
    // pub mod api;
    // #[path = "google.rpc.rs"]
    // pub mod rpc;
    // #[path = "google.cloud.sql.v1.rs"]
    // pub mod cloud;
    // #[path = "google.monitoring.v3.rs"]
    // pub mod monitoring;
}

use self::google::logging::v2::{log_entry, LogEntry};
use crate::{
    error::{GCloudError, WKError},
    WKClient,
};
use chrono::Duration;
use google::logging::v2::{
    logging_service_v2_client::LoggingServiceV2Client, ListLogEntriesRequest,
};
use google_cloud_monitoring::{
    v3::{MetricServiceClient, ListTimeSeriesRequest, TimeInterval},
    MetricKind, ValueType,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tonic::{metadata::MetadataValue, transport::Channel, Request};

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

pub struct CloudSqlInstances {
    // pub instances: Option<Vec<google::cloud::sql::v1::DatabaseInstance>>,
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
}

/// Functions from Google Cloud service.
impl WKClient {
    /// Get log entries from Google Cloud.
    pub async fn get_gcloud_log_entries(
        &self,
        optons: LogEntriesOptions,
        access_token: String,
    ) -> Result<LogEntries, WKError> {
        let google_client = GCloudClient::new(access_token);
        google_client
            .get_log_entries(optons)
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



    // Get SQL status entries from Google Cloud.
    pub async fn get_gcloud_sql_instances_metrics(
        &self,
        access_token: String,
    ) -> Result<(), WKError> {
        let project_id = "your-project-id";
        let metric_service_client = MetricServiceClient::new();

        let request = Request::new(ListTimeSeriesRequest {
            name: format!("projects/{}", project_id),
            filter: format!(
                r#"metric.type="compute.googleapis.com/instance/cpu/utilization" OR metric.type="compute.googleapis.com/instance/memory/utilization""#
            ),
            interval: Some(TimeInterval {
                end_time: Some(prost_types::Timestamp {
                    seconds: chrono::Utc::now().timestamp(),
                    nanos: 0,
                }),
                start_time: Some(prost_types::Timestamp {
                    seconds: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(),
                    nanos: 0,
                }),
            }),
            view: 2,
            ..Default::default()
        });

        let response = metric_service_client.list_time_series(request).await?;

        for time_series in response.into_inner().time_series {
            let metric_kind = time_series.metric_kind.unwrap_or_default();
            let value_type = time_series.value_type.unwrap_or_default();
            let metric_name = time_series.metric.unwrap_or_default().type_;

            match (metric_kind, value_type) {
                (MetricKind::Gauge, ValueType::Double) => {
                    let points = time_series.points.unwrap_or_default();
                    for point in points {
                        let value = point.value.unwrap_or_default().double_value;
                        println!("Metric: {}, Value: {}", metric_name, value);
                    }
                }
                _ => {
                    println!("Unsupported metric kind or value type");
                }
            }
        }

        Ok(())
    }
}