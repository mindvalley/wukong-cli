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
use config::{Config, CONFIG_FILE};
use error::{APIError, CliError};
use graphql::{applications_query, post_graphql, ApplicationsQuery};
use graphql_client::{GraphQLQuery, Response};
use hyper::header;

pub async fn run() -> Result<bool, CliError> {
    let app = App::new()?;

    app.cli.execute().await
}

pub trait WKConfig {
    fn api_url(&self) -> String;
    fn access_token(&self) -> Option<String>;
}

pub struct WKClient {
    pub(crate) api_url: String,
    pub(crate) access_token: Option<String>,
}

impl WKClient {
    pub fn new(config: &impl WKConfig) -> Self {
        Self {
            api_url: config.api_url(),
            access_token: config.access_token(),
        }
    }

    pub async fn fetch_applications(
        &self,
        variables: applications_query::Variables,
    ) -> Result<applications_query::ResponseData, APIError> {
        let mut headers = header::HeaderMap::new();

        if let Some(token) = self.access_token.as_ref() {
            let auth_value = format!("Bearer {}", token);
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&auth_value).unwrap(),
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Self::post_graphql::<ApplicationsQuery, _>(&client, &self.api_url, variables).await
    }

    async fn post_graphql<Q, U>(
        client: &reqwest::Client,
        // client: &WKClient,
        url: U,
        variables: Q::Variables,
    ) -> Result<Q::ResponseData, APIError>
    where
        Q: GraphQLQuery,
        U: reqwest::IntoUrl,
    {
        let body = Q::build_query(variables);
        let res: Response<Q::ResponseData> =
            client.post(url).json(&body).send().await?.json().await?;

        todo!()
        // if let Some(errors) = res.errors {
        //     if errors[0].message.to_lowercase().contains("not authorized") {
        //         // Handle unauthorized errors in a custom way
        //         Err(RailwayError::Unauthorized)
        //     } else {
        //         Err(RailwayError::GraphQLError(errors[0].message.clone()))
        //     }
        // } else if let Some(data) = res.data {
        //     Ok(data)
        // } else {
        //     Err(RailwayError::MissingResponseData)
        // }
    }
}
