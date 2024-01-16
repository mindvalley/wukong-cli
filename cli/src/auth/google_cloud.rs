use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};
use time::{format_description, OffsetDateTime};
use tonic::async_trait;
use yup_oauth2::{
    authenticator_delegate::{DefaultInstalledFlowDelegate, InstalledFlowDelegate},
    hyper, hyper_rustls,
    storage::{TokenInfo, TokenStorage},
    ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JSONToken {
    scopes: Vec<String>,
    token: TokenInfo,
}

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

const GOOGLE_CLIENT_ID: &str =
    "16448589901-ccrhj03hhg6adn9uv8vi2trnpmd62k6n.apps.googleusercontent.com";
const GOOGLE_CLIENT_SECRET: &str = "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";
const TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
const AUTH_URI: &str = "https://accounts.google.com/o/oauth2/auth";
const REDIRECT_URI: &str = "http://127.0.0.1/8855";
const AUTH_PROVIDER_X509_CERT_URL: &str = "https://www.googleapis.com/oauth2/v1/certs";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct GoogleCloudConfig {
    /// used when authorizing calls to oauth2 enabled services.
    pub access_token: String,
    /// used to refresh an expired access_token.
    pub refresh_token: String,
    /// The time when the token expires.
    pub expiry_time: String,
    /// Optionally included by the OAuth2 server and may contain information to verify the identity
    /// used to obtain the access token.
    /// Specifically Google API:s include this if the additional scopes "email" and/or "profile"
    /// are used. In that case the content is an JWT token.
    pub id_token: Option<String>,
}

struct ConfigTokenStore {
    config: Config,
}

#[async_trait]
impl TokenStorage for ConfigTokenStore {
    async fn set(&self, _scopes: &[&str], token: TokenInfo) -> anyhow::Result<()> {
        let mut config = self.config.clone();

        config.auth.google_cloud = Some(GoogleCloudConfig {
            access_token: token.access_token.expect("Invalid access token"),
            refresh_token: token.refresh_token.expect("Invalid refresh token"),
            expiry_time: token
                .expires_at
                .expect("Invalid expiry time")
                .format(&format_description::well_known::Rfc3339)?,
            id_token: token.id_token,
        });

        config.save_to_default_path()?;

        Ok(())
    }

    async fn get(&self, _target_scopes: &[&str]) -> Option<TokenInfo> {
        let google_cloud = self.config.auth.google_cloud.clone()?;

        Some(TokenInfo {
            access_token: Some(google_cloud.access_token),
            refresh_token: Some(google_cloud.refresh_token),
            expires_at: Some(
                OffsetDateTime::parse(
                    &google_cloud.expiry_time,
                    &format_description::well_known::Rfc3339,
                )
                .expect("Invalid expiry time"),
            ),
            id_token: google_cloud.id_token,
        })
    }
}

pub async fn get_token_or_login(config: Option<Config>) -> String {
    let secret = ApplicationSecret {
        client_id: GOOGLE_CLIENT_ID.to_string(),
        client_secret: GOOGLE_CLIENT_SECRET.to_string(),
        token_uri: TOKEN_URI.to_string(),
        auth_uri: AUTH_URI.to_string(),
        redirect_uris: vec![REDIRECT_URI.to_string()],
        project_id: None,
        client_email: None,
        auth_provider_x509_cert_url: Some(AUTH_PROVIDER_X509_CERT_URL.to_string()),
        client_x509_cert_url: None,
    };

    let client = hyper::Client::builder().build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .enable_http2()
            .build(),
    );

    let config = match config {
        Some(config) => config,
        None => Config::load_from_default_path().expect("Unable to load config"),
    };

    let authenticator = InstalledFlowAuthenticator::with_client(
        secret,
        InstalledFlowReturnMethod::HTTPPortRedirect(8855),
        client,
    )
    .with_storage(Box::new(ConfigTokenStore { config }))
    .flow_delegate(Box::new(InstalledFlowBrowserDelegate))
    .build()
    .await
    .unwrap();

    authenticator
        .token(&["https://www.googleapis.com/auth/logging.read"])
        .await
        .unwrap()
        .token()
        .unwrap()
        .to_string()
}

pub async fn get_access_token() -> Option<String> {
    let config = match Config::load_from_default_path() {
        Ok(config) => config,
        Err(_) => return None,
    };

    // Sometimes access token exist but is expired, so call get_token_or_login() to refresh it
    // before returning it.
    if config.auth.google_cloud.is_some() {
        Some(get_token_or_login(None).await)
    } else {
        None
    }
}
