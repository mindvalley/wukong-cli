use chrono::{offset::Utc, SecondsFormat};
#[cfg(all(feature = "prod"))]
use once_cell::sync::Lazy;
#[cfg(all(feature = "prod"))]
use reqwest::header;
use serde::{Deserialize, Serialize};

#[cfg(all(feature = "prod"))]
const EVENT_THRESHOLD: usize = 20;

#[cfg(all(feature = "prod"))]
const HONEYCOMB_API_KEY: &'static str = env!("WUKONG_HONEYCOMB_API_KEY");

#[cfg(all(feature = "prod"))]
const HONEYCOMB_DATASET: &'static str = "wukong_telemetry_prod";

/// The default path to the wukong telemetry file.
///
/// This is a [Lazy] of `Option<String>`, the value of which is
///
/// > `~/.config/wukong/telemetry.yml`
///
/// It will only be `None` if it is unable to identify the user's home
/// directory, which should not happen under typical OS environments.
///
/// [Lazy]: https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html
#[cfg(all(feature = "prod"))]
pub static TELEMETRY_FILE: Lazy<Option<String>> = Lazy::new(|| {
    dirs::home_dir().map(|mut path| {
        path.extend([".config", "wukong", "telemetry.json"]);
        path.to_str().unwrap().to_string()
    })
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelemetryData {
    timestamp: String,
    actor: String,
    application: Option<String>,
    #[serde(flatten)]
    event: TelemetryEvent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum TelemetryEvent {
    Command {
        #[serde(rename = "cmd_name")]
        name: String,
        #[serde(rename = "cmd_run_mode")]
        run_mode: CommandRunMode,
    },
    Api {
        #[serde(rename = "api_name")]
        name: String,
        #[serde(rename = "api_duration")]
        duration: u64,
        #[serde(rename = "api_response")]
        response: APIResponse,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HoneycombEventData {
    time: String,
    data: TelemetryData,
}

impl From<TelemetryData> for HoneycombEventData {
    fn from(telemetry_data: TelemetryData) -> Self {
        Self {
            time: telemetry_data.timestamp.clone(),
            data: telemetry_data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum CommandRunMode {
    Interactive,
    NonInteractive,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum APIResponse {
    Success,
    Error,
}

impl TelemetryData {
    pub fn new(event: TelemetryEvent, application: Option<String>, actor: String) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            actor,
            application,
            event,
        }
    }

    pub async fn record_event(&self) {
        #[cfg(all(feature = "prod"))]
        {
            let telemetry_file = TELEMETRY_FILE
                .as_ref()
                .expect("Unable to identify user's home directory");

            let mut telemetry_data = {
                let read_result = std::fs::read_to_string(telemetry_file);
                if let Ok(data) = read_result {
                    serde_json::from_str::<Vec<TelemetryData>>(&data).unwrap()
                } else {
                    Vec::new()
                }
            };

            telemetry_data.push(self.clone());

            // if telemetry_data is more than the EVENT_THRESHOLD, then send the events in batch to honeycomb
            if telemetry_data.len() >= EVENT_THRESHOLD {
                let event_data: Vec<HoneycombEventData> =
                    telemetry_data.into_iter().map(|each| each.into()).collect();

                send_event(event_data).await;

                std::fs::write(telemetry_file, "[]").unwrap();
            } else {
                std::fs::write(
                    telemetry_file,
                    serde_json::to_string_pretty(&telemetry_data).unwrap(),
                )
                .unwrap();
            }
        }
    }
}

#[cfg(all(feature = "prod"))]
async fn send_event(event_data: Vec<HoneycombEventData>) {
    let client = reqwest::Client::builder().build().unwrap();
    let url = format!("https://api.honeycomb.io/1/batch/{}", HONEYCOMB_DATASET);

    let _ = client
        .post(url)
        .header("X-Honeycomb-Team", HONEYCOMB_API_KEY)
        .header(header::CONTENT_TYPE, "application/json")
        .json(&event_data)
        .send()
        .await;
}
