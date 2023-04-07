use std::{future::Future, pin::Pin};

use crate::error::GCloudError;
use chrono::Duration;
use google_logging2::{
    api::{ListLogEntriesRequest, TailLogEntriesRequest},
    hyper, hyper_rustls,
    oauth2::{
        authenticator_delegate::{DefaultInstalledFlowDelegate, InstalledFlowDelegate},
        ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
    },
    Logging,
};

/// async function to be pinned by the `present_user_url` method of the trait
/// we use the existing `DefaultInstalledFlowDelegate::present_user_url` method as a fallback for
/// when the browser did not open for example, the user still see's the URL.
async fn browser_user_url(url: &str, need_code: bool) -> Result<String, String> {
    if webbrowser::open(url).is_ok() {
        println!("webbrowser was successfully opened.");
    }
    let def_delegate = DefaultInstalledFlowDelegate;
    def_delegate.present_user_url(url, need_code).await
}

/// our custom delegate struct we will implement a flow delegate trait for:
/// in this case we will implement the `InstalledFlowDelegated` trait
#[derive(Copy, Clone)]
struct InstalledFlowBrowserDelegate;

/// here we implement only the present_user_url method with the added webbrowser opening
/// the other behaviour of the trait does not need to be changed.
impl InstalledFlowDelegate for InstalledFlowBrowserDelegate {
    /// the actual presenting of URL and browser opening happens in the function defined above here
    /// we only pin it
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(browser_user_url(url, need_code))
    }
}

pub struct LogEntriesOptions {
    pub project_ids: Option<Vec<String>>,
    pub filter: Option<String>,
    pub page_size: Option<i32>,
    pub page_token: Option<String>,
    pub order_by: Option<String>,
    pub resource_names: Option<Vec<String>>,
}

pub struct LogEntriesTailOptions {
    pub filter: Option<String>,
    pub buffer_window: Option<Duration>,
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
    const GOOGLE_CLIENT_ID: &'static str =
        "16448589901-ccrhj03hhg6adn9uv8vi2trnpmd62k6n.apps.googleusercontent.com";
    const GOOGLE_CLIENT_SECRET: &'static str = "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";
    const TOKEN_URI: &'static str = "https://oauth2.googleapis.com/token";
    const AUTH_URI: &'static str = "https://accounts.google.com/o/oauth2/auth";
    const REDIRECT_URI: &'static str = "http://127.0.0.1/8855";
    const AUTH_PROVIDER_X509_CERT_URL: &'static str = "https://www.googleapis.com/oauth2/v1/certs";

    pub async fn new() -> Result<Self, GCloudError> {
        let secret = ApplicationSecret {
            client_id: Self::GOOGLE_CLIENT_ID.to_string(),
            client_secret: Self::GOOGLE_CLIENT_SECRET.to_string(),
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
        .flow_delegate(Box::new(InstalledFlowBrowserDelegate))
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
        options: LogEntriesOptions,
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

    pub async fn get_log_entries_tail(
        &self,
        options: LogEntriesTailOptions,
    ) -> Result<(), GCloudError> {
        let request = TailLogEntriesRequest {
            filter: options.filter,
            buffer_window: options.buffer_window,
            resource_names: options.resource_names,
        };
        let call = self.hub.entries().tail(request);
        let (response, output_schema) = call
            .add_scope("https://www.googleapis.com/auth/logging.read")
            .doit()
            .await?;

        println!("{:#?}", output_schema);
        println!("{:#?}", response);

        Ok(())
    }
}
