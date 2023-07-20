mod app;
mod auth;
mod commands;
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
mod telemetry;
mod utils;

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

pub struct OktaAuthenticator {
    okta_id: String,
    callback_url: String,
}
pub struct OktaAuthenticatorBuilder {
    okta_id: String,
    callback_url: String,
}
pub struct OktaAuthResponse {
    pub account: String,
    pub subject: String,
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expiry_time: String,
}
pub struct OktaRefreshTokensResponse {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expiry_time: String,
}
impl OktaAuthenticator {
    pub fn builder() -> OktaAuthenticatorBuilder {
        OktaAuthenticatorBuilder {
            okta_id: "".to_string(),
            callback_url: "http://localhost:6758/login/callback".to_string(),
        }
    }

    pub async fn login(&self) -> Result<OktaAuthResponse, AuthError> {
        let resp = Auth::new(&self.okta_id).login().await?;
        Ok(OktaAuthResponse {
            account: resp.account,
            subject: resp.subject,
            id_token: resp.id_token,
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expiry_time: resp.expiry_time,
        })
    }

    pub async fn refresh_tokens(
        &self,
        refresh_token: String,
    ) -> Result<OktaRefreshTokensResponse, AuthError> {
        let resp = Auth::new(&self.okta_id)
            .refresh_tokens(&RefreshToken::new(refresh_token))
            .await?;
        Ok(OktaRefreshTokensResponse {
            id_token: resp.id_token,
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expiry_time: resp.expiry_time,
        })
    }
}

impl OktaAuthenticatorBuilder {
    #[must_use]
    pub fn with_okta_id(mut self, okta_id: &str) -> Self {
        self.okta_id = okta_id.to_string();
        self
    }

    pub fn with_callback_url(mut self, callback_url: &str) -> Self {
        self.callback_url = callback_url.to_string();
        self
    }

    pub fn build(self) -> OktaAuthenticator {
        OktaAuthenticator {
            okta_id: self.okta_id,
            callback_url: self.callback_url,
        }
    }
}

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
