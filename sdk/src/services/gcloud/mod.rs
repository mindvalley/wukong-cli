#[rustfmt::skip]
#[path = "api"]
pub mod google {
    #[path = ""]
    pub mod logging {
        #[path = "google.logging.r#type.rs"]
        pub mod r#type;
        #[path = "google.logging.v2.rs"]
        pub mod v2;
    }
    #[path = "google.api.rs"]
    pub mod api;
    #[path = "google.rpc.rs"]
    pub mod rpc;
}

use crate::{
    error::{GCloudError, WKError},
    WKClient,
};
use chrono::Duration;
use google::logging::v2::{
    logging_service_v2_client::LoggingServiceV2Client, ListLogEntriesRequest,
};
use std::{future::Future, pin::Pin};
use tonic::{metadata::MetadataValue, transport::Channel, Request};
use yup_oauth2::{
    authenticator_delegate::{DefaultInstalledFlowDelegate, InstalledFlowDelegate},
    ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
};

/// async function to be pinned by the `present_user_url` method of the trait
/// we use the existing `DefaultInstalledFlowDelegate::present_user_url` method as a fallback for
/// when the browser did not open for example, the user still see's the URL.
async fn browser_user_url(url: &str, need_code: bool) -> Result<String, String> {
    let url = format!("{}&prompt=consent", url);
    if webbrowser::open(&url).is_ok() {
        println!("Your browser has been opened to visit:\n\n\t{url}\n");
        Ok(String::new())
    } else {
        let def_delegate = DefaultInstalledFlowDelegate;
        def_delegate.present_user_url(&url, need_code).await
    }
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

#[derive(Debug, Default)]
pub struct LogEntriesOptions {
    pub project_ids: Option<Vec<String>>,
    pub filter: Option<String>,
    pub page_size: Option<i32>,
    pub page_token: Option<String>,
    pub order_by: Option<String>,
    pub resource_names: Option<Vec<String>>,
}

impl From<LogEntriesOptions> for ListLogEntriesRequest {
    fn from(value: LogEntriesOptions) -> Self {
        ListLogEntriesRequest {
            filter: value.filter.unwrap_or_default(),
            page_size: value.page_size.unwrap_or_default(),
            page_token: value.page_token.unwrap_or_default(),
            order_by: value.order_by.unwrap_or_default(),
            resource_names: value.resource_names.unwrap_or_default(),
        }
    }
}

pub struct LogEntriesTailOptions {
    pub filter: Option<String>,
    pub buffer_window: Option<Duration>,
    pub resource_names: Option<Vec<String>>,
}

pub struct LogEntries {
    pub entries: Option<Vec<google::logging::v2::LogEntry>>,
    pub next_page_token: Option<String>,
}

pub struct GCloudClient {
    access_token: String,
}

impl GCloudClient {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
    // const GOOGLE_CLIENT_ID: &'static str =
    //     "16448589901-ccrhj03hhg6adn9uv8vi2trnpmd62k6n.apps.googleusercontent.com";
    // const GOOGLE_CLIENT_SECRET: &'static str = "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";
    // const TOKEN_URI: &'static str = "https://oauth2.googleapis.com/token";
    // const AUTH_URI: &'static str = "https://accounts.google.com/o/oauth2/auth";
    // const REDIRECT_URI: &'static str = "http://127.0.0.1/8855";
    // const AUTH_PROVIDER_X509_CERT_URL: &'static str = "https://www.googleapis.com/oauth2/v1/certs";

    // pub async fn new() -> Self {
    //     let secret = ApplicationSecret {
    //         client_id: Self::GOOGLE_CLIENT_ID.to_string(),
    //         client_secret: Self::GOOGLE_CLIENT_SECRET.to_string(),
    //         token_uri: Self::TOKEN_URI.to_string(),
    //         auth_uri: Self::AUTH_URI.to_string(),
    //         redirect_uris: vec![Self::REDIRECT_URI.to_string()],
    //         project_id: None,
    //         client_email: None,
    //         auth_provider_x509_cert_url: Some(Self::AUTH_PROVIDER_X509_CERT_URL.to_string()),
    //         client_x509_cert_url: None,
    //     };
    //
    //     // ~/.config/wukong/
    //     #[cfg(feature = "prod")]
    //     let config_dir = dirs::home_dir()
    //         .map(|mut path| {
    //             path.extend([".config", "wukong"]);
    //             path.to_str().unwrap().to_string()
    //         })
    //         .expect("wukong config path is invalid");
    //
    //     #[cfg(not(feature = "prod"))]
    //     let config_dir = {
    //         match std::env::var("WUKONG_DEV_GCLOUD_FILE") {
    //             Ok(config) => config,
    //             Err(_) => dirs::home_dir()
    //                 .map(|mut path| {
    //                     path.extend([".config", "wukong", "dev"]);
    //                     path.to_str().unwrap().to_string()
    //                 })
    //                 .expect("wukong dev config path is invalid"),
    //         }
    //     };
    //
    //     let client = hyper::Client::builder().build(
    //         hyper_rustls::HttpsConnectorBuilder::new()
    //             .with_native_roots()
    //             .https_only()
    //             .enable_http1()
    //             .enable_http2()
    //             .build(),
    //     );
    //
    //     let authenticator = InstalledFlowAuthenticator::with_client(
    //         secret,
    //         InstalledFlowReturnMethod::HTTPPortRedirect(8855),
    //         client,
    //     )
    //     .persist_tokens_to_disk(format!("{}/gcloud_logging", config_dir))
    //     .flow_delegate(Box::new(InstalledFlowBrowserDelegate))
    //     .build()
    //     .await
    //     .unwrap();
    //
    //     let access_token = authenticator
    //         .token(&["https://www.googleapis.com/auth/logging.read"])
    //         .await
    //         .unwrap();
    //
    //     Self {
    //         access_token: access_token.token().unwrap().to_string(),
    //     }
    // }

    pub async fn get_log_entries(
        &self,
        options: LogEntriesOptions,
    ) -> Result<LogEntries, GCloudError> {
        let bearer_token = format!("Bearer {}", self.access_token);
        let header_value: MetadataValue<_> = bearer_token.parse().unwrap();

        let channel = Channel::from_static("https://logging.googleapis.com")
            .connect()
            .await
            .unwrap();

        let mut service =
            LoggingServiceV2Client::with_interceptor(channel, move |mut req: Request<()>| {
                let metadata_map = req.metadata_mut();
                metadata_map.insert("authorization", header_value.clone());
                metadata_map.insert("user-agent", "grpc-go/1.14".parse().unwrap());

                Ok(req)
            });

        let request: ListLogEntriesRequest = options.into();

        let response = service
            .list_log_entries(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        Ok(LogEntries {
            entries: Some(response.entries),
            next_page_token: Some(response.next_page_token),
        })
    }
}

impl WKClient {
    pub async fn get_gcloud_log_entries(
        &self,
        optons: LogEntriesOptions,
        access_token: String,
    ) -> Result<LogEntries, WKError> {
        let google_client = GCloudClient::new(access_token);
        google_client
            .get_log_entries(optons)
            .await
            .map_err(|err| err.into())
    }
}
