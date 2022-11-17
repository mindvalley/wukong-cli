use std::collections::HashMap;

use chrono::offset::Utc;
use libhoney::{json, FieldHolder, Value};

#[derive(Debug)]
pub struct TelemetryData {
    timestamp: i64,
    actor: String,
    application: Option<String>,
    command: Option<Command>,
    api_call: Option<ApiCall>,
}

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub run_mode: CommandRunMode,
}

#[derive(Debug)]
pub struct ApiCall {
    duration: i64,
    response: APIResponse,
}

macro_rules! enum_str {
    (pub enum $name:ident {
        $($variant:ident = $val:expr),*,
    }) => {
        #[derive(Debug)]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            fn name(&self) -> &'static str {
                match self {
                    $($name::$variant => $val),*
                }
            }
        }
    };
}

enum_str! {
    pub enum CommandRunMode {
        Interactive = "interactive",
        NonInteractive = "non-interactive",
    }
}

enum_str! {
    pub enum APIResponse {
        Success = "Success",
        Error = "Error",
    }
}

impl TelemetryData {
    pub fn new(command: Command) -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            actor: "Sjklsdfhkfhj".to_string(),
            application: None,
            command: Some(command),
            api_call: None,
        }
    }

    pub async fn send_event(&self) {
        let mut client = libhoney::init(libhoney::Config {
            options: libhoney::client::Options {
                dataset: "wukong_telemetry_dev".to_string(),
                ..libhoney::client::Options::default()
            },
            transmission_options: libhoney::transmission::Options::default(),
        });

        // let mut data: HoneycombData = *self.into();
        // data.insert("timestamp".to_string(), json!(153.12));
        // data.insert("duration_ms".to_string(), json!(153.12));
        // data.insert("method".to_string(), Value::String("get".to_string()));
        // data.insert(
        //     "hostname".to_string(),
        //     Value::String("appserver15".to_string()),
        // );
        // data.insert("payload_length".to_string(), json!(27));
        let mut data = HashMap::new();
        data.insert("timestamp".to_string(), json!(self.timestamp));
        data.insert("actor".to_string(), Value::String(self.actor.clone()));
        data.insert(
            "application".to_string(),
            match &self.application {
                Some(application) => Value::String(application.to_string()),
                None => Value::Null,
            },
        );
        match &self.command {
            Some(command) => {
                data.insert("cmd_name".to_string(), Value::String(command.name.clone()));
                data.insert(
                    "cmd_run_mode".to_string(),
                    Value::String(command.run_mode.name().to_string()),
                );
            }
            None => {
                data.insert("cmd_name".to_string(), Value::Null);
                data.insert("cmd_run_mode".to_string(), Value::Null);
            }
        }

        let mut ev = client.new_event();
        ev.add(data);

        match ev.send(&mut client) {
            Ok(()) => {
                let response = client.responses().iter().next().unwrap();
                assert_eq!(response.error, None);
            }
            Err(e) => {
                log::error!("Could not send event: {}", e);
            }
        }

        client.close().unwrap();
    }
}

// impl Into<HoneycombData> for TelemetryData {
//     fn into(self) -> HoneycombData {
//         let mut hash_map = HashMap::new();
//         hash_map.insert("timestamp".to_string(), json!(self.timestamp));
//         hash_map.insert("actor".to_string(), Value::String(self.actor));
//         hash_map.insert(
//             "application".to_string(),
//             match self.application {
//                 Some(application) => Value::String(application),
//                 None => Value::Null,
//             },
//         );

//         hash_map
//     }
// }
