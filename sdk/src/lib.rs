mod app;
mod auth;
mod config;
pub mod error;
pub mod graphql;
mod loader;
mod logger;
pub mod output;
pub mod services {
    pub mod gcloud;
    pub mod vault;
}
pub mod linter;
mod telemetry;
mod utils;

pub use utils::secret_extractors;

use app::App;
use auth::Auth;
use config::{Config, CONFIG_FILE};
use error::{APIError, AuthError, WKError};
use graphql::{
    applications_query, pipeline_query,
    pipelines_query::{self, PipelinesQueryPipelines},
    ApplicationsQuery, GQLClient, PipelinesQuery,
};
use graphql_client::{GraphQLQuery, Response};
use hyper::header;
use openidconnect::RefreshToken;

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
