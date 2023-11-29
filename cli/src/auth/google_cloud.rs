use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};
use yup_oauth2::{
    authenticator_delegate::{DefaultInstalledFlowDelegate, InstalledFlowDelegate},
    hyper, hyper_rustls,
    storage::TokenInfo,
    ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JSONToken {
    scopes: Vec<String>,
    token: TokenInfo,
}

pub static CONFIG_PATH: Lazy<Option<String>> = Lazy::new(|| {
    #[cfg(feature = "prod")]
    return dirs::home_dir().map(|mut path| {
        path.extend([".config", "wukong"]);
        path.to_str().unwrap().to_string()
    });

    #[cfg(not(feature = "prod"))]
    {
        match std::env::var("WUKONG_DEV_GCLOUD_FILE") {
            Ok(config) => Some(config),
            Err(_) => dirs::home_dir().map(|mut path| {
                path.extend([".config", "wukong", "dev"]);
                path.to_str().unwrap().to_string()
            }),
        }
    }
});

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

pub async fn get_token_or_login() -> String {
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

    let authenticator = InstalledFlowAuthenticator::with_client(
        secret,
        InstalledFlowReturnMethod::HTTPPortRedirect(8855),
        client,
    )
    .persist_tokens_to_disk(format!(
        "{}/gcloud_logging",
        CONFIG_PATH
            .as_ref()
            .expect("Unable to identify user's home directory"),
    ))
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
    let contents = tokio::fs::read(format!(
        "{}/gcloud_logging",
        CONFIG_PATH
            .as_ref()
            .expect("Unable to identify user's home directory")
    ))
    .await;

    let tokens = contents
        .map(|contents| {
            serde_json::from_slice::<Vec<JSONToken>>(&contents)
                .map_err(|_| {
                    eprintln!("Failed to parse token file.");
                })
                .ok()
        })
        .unwrap_or(None);

    let json_token = tokens.and_then(|tokens| {
        tokens
            .iter()
            .find(|token| {
                token
                    .scopes
                    .contains(&"https://www.googleapis.com/auth/logging.read".to_string())
            })
            .map(|token| token.token.access_token.clone())
    });

    // Sometimes access token exist but is expired, so call get_token_or_login() to refresh it
    // before returning it.
    if json_token.is_some() {
        Some(get_token_or_login().await)
    } else {
        None
    }
}
