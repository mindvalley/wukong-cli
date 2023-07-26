mod app;
mod config;
pub mod error;
pub mod graphql;
mod loader;
mod logger;
pub mod services {
    pub mod gcloud;
    pub mod vault;
}
pub mod linter;
pub mod telemetry;
mod utils;

pub mod wk_telemetry {
    pub use crate::telemetry::*;
    pub use wukong_telemetry_macro::wukong_telemetry;
}

pub use utils::secret_extractors;

pub struct WKConfig {
    pub api_url: String,
    pub access_token: Option<String>,
}

pub struct WKClient {
    pub(crate) api_url: String,
    pub(crate) access_token: Option<String>,
}

impl WKClient {
    pub fn new(config: WKConfig) -> Self {
        Self {
            api_url: config.api_url,
            access_token: config.access_token,
        }
    }
}
