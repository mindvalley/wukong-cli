pub mod error;
pub mod graphql;
pub mod services {
    pub mod gcloud;
    pub mod vault;
}
pub mod linter;
mod utils;

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
