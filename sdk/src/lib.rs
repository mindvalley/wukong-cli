pub mod error;
pub mod graphql;
pub mod services {
    pub mod gcloud;
    pub mod vault;
}
pub mod linter;
mod utils;

pub use utils::secret_extractors;

#[derive(Debug, Default)]
pub enum ApiChannel {
    Canary,
    #[default]
    Stable,
}

pub struct WKConfig {
    pub api_url: String,
    pub access_token: String,
    pub channel: ApiChannel,
}

pub struct WKClient {
    pub(crate) api_url: String,
    pub(crate) access_token: String,
    pub(crate) channel: ApiChannel,
}

impl WKClient {
    pub fn new(config: WKConfig) -> Self {
        Self {
            api_url: config.api_url,
            access_token: config.access_token,
            channel: config.channel,
        }
    }

    pub fn set_access_token(&mut self, token: String) {
        self.access_token = token;
    }

    pub fn set_api_url(&mut self, url: String) {
        self.api_url = url;
    }

    pub fn set_channel(&mut self, channel: ApiChannel) {
        self.channel = channel;
    }
}
