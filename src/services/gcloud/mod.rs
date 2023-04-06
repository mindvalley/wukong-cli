use crate::error::GCloudError;
use google_logging2::{
    api::ListLogEntriesRequest,
    hyper, hyper_rustls,
    oauth2::{ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod},
    Logging,
};
use std::env;

pub struct LogEntriesOption {
    pub project_ids: Option<Vec<String>>,
    pub filter: Option<String>,
    pub page_size: Option<i32>,
    pub page_token: Option<String>,
    pub order_by: Option<String>,
    pub resource_names: Option<Vec<String>>,
}

pub struct LogEntries {
    pub entries: Option<Vec<google_logging2::api::LogEntry>>,
    pub next_page_token: Option<String>,
}

pub struct GCloudClient {
    hub: Logging<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl GCloudClient {
    const TOKEN_URI: &'static str = "https://oauth2.googleapis.com/token";
    const AUTH_URI: &'static str = "https://accounts.google.com/o/oauth2/auth";
    const REDIRECT_URI: &'static str = "http://127.0.0.1/8855";
    const AUTH_PROVIDER_X509_CERT_URL: &'static str = "https://www.googleapis.com/oauth2/v1/certs";

    pub async fn new() -> Result<Self, GCloudError> {
        let secret = ApplicationSecret {
            client_id: env::var("GOOGLE_CLIENT_ID")
                .expect("Missing the GOOGLE_CLIENT_ID environment variable."),
            client_secret: env::var("GOOGLE_CLIENT_SECRET")
                .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
            token_uri: Self::TOKEN_URI.to_string(),
            auth_uri: Self::AUTH_URI.to_string(),
            redirect_uris: vec![Self::REDIRECT_URI.to_string()],
            project_id: None,
            client_email: None,
            auth_provider_x509_cert_url: Some(Self::AUTH_PROVIDER_X509_CERT_URL.to_string()),
            client_x509_cert_url: None,
        };

        // ~/.config/wukong/
        #[cfg(all(feature = "prod"))]
        let config_dir = dirs::home_dir()
            .map(|mut path| {
                path.extend([".config", "wukong"]);
                path.to_str().unwrap().to_string()
            })
            .expect("wukong config path is invalid");

        #[cfg(not(feature = "prod"))]
        let config_dir = {
            match std::env::var("WUKONG_DEV_GCLOUD_FILE") {
                Ok(config) => config,
                Err(_) => dirs::home_dir()
                    .map(|mut path| {
                        path.extend([".config", "wukong", "dev"]);
                        path.to_str().unwrap().to_string()
                    })
                    .expect("wukong dev config path is invalid"),
            }
        };

        let auth = InstalledFlowAuthenticator::builder(
            secret,
            InstalledFlowReturnMethod::HTTPPortRedirect(8855),
        )
        .persist_tokens_to_disk(format!("{}/gcloud_logging", config_dir))
        .build()
        .await?;

        let hub = Logging::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
            auth,
        );

        Ok(Self { hub })
    }

    pub async fn get_log_entries(
        &self,
        options: LogEntriesOption,
    ) -> Result<LogEntries, GCloudError> {
        let request = ListLogEntriesRequest {
            filter: options.filter,
            order_by: options.order_by,
            page_size: options.page_size.or(Some(10)),
            page_token: options.page_token,
            project_ids: options.project_ids,
            resource_names: options.resource_names,
        };
        let call = self.hub.entries().list(request);
        let (_response, output_schema) = call
            .add_scope("https://www.googleapis.com/auth/logging.read")
            .doit()
            .await?;

        Ok(LogEntries {
            entries: output_schema.entries,
            next_page_token: output_schema.next_page_token,
        })
    }
}
