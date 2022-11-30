use chrono::{offset::Utc, SecondsFormat};
use lazy_static::lazy_static;
use reqwest::header;
use serde::{Deserialize, Serialize};

const EVENT_THRESHOLD: usize = 20;

lazy_static! {
    /// The default path to the wukong telemetry file.
    ///
    /// This is a [lazy_static] of `Option<String>`, the value of which is
    ///
    /// > `~/.config/wukong/telemetry.json`
    ///
    /// It will only be `None` if it is unable to identify the user's home
    /// directory, which should not happen under typical OS environments.
    ///
    /// [lazy_static]: https://docs.rs/lazy_static
    pub static ref TELEMETRY_FILE: Option<String> = {
        dirs::home_dir().map(|mut path| {
            path.extend([".config", "wukong", "telemetry.json"]);
            path.to_str().unwrap().to_string()
        })
    };
}

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
        let telemetry_file = TELEMETRY_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let mut telemetry_data = {
            let read_result = std::fs::read_to_string(&telemetry_file);
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

async fn send_event(event_data: Vec<HoneycombEventData>) {
    let client = reqwest::Client::builder().build().unwrap();

    // println!("event data: {:?}", &event_data);
    println!("event data: {:#?}", serde_json::to_string(&event_data));

    let resp = client
        .post("https://api.honeycomb.io/1/batch/wukong_telemetry_dev")
        .header(header::CONTENT_TYPE, "application/json")
        .json(&event_data)
        .send()
        .await
        .unwrap();

    println!("resp: {:?}", resp);
}
