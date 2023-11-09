use std::{future::Future, pin::Pin};
use yup_oauth2::{
    authenticator_delegate::{DefaultInstalledFlowDelegate, InstalledFlowDelegate},
    hyper, hyper_rustls, ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
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
    // panic!("This is a custom panic message");

    // ~/.config/wukong/
    #[cfg(feature = "prod")]
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
    .persist_tokens_to_disk(format!("{}/gcloud_logging", config_dir))
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
