use std::{
    env,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

use chrono::{Duration, Utc};
use google_logging2::{
    api::{ListLogEntriesRequest, ListLogEntriesResponse},
    hyper, hyper_rustls,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse,
};
use url::Url;

pub struct GCloudClient {}

impl GCloudClient {
    pub async fn login() {
        let google_client_id = ClientId::new(
            env::var("GOOGLE_CLIENT_ID")
                .expect("Missing the GOOGLE_CLIENT_ID environment variable."),
        );
        let google_client_secret = ClientSecret::new(
            env::var("GOOGLE_CLIENT_SECRET")
                .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
        );
        let auth_url =
            oauth2::AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())
                .expect("Invalid authorization endpoint URL");
        let token_url = oauth2::TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
            .expect("Invalid token endpoint URL");

        // Set up the config for the Google OAuth2 process.
        let client = BasicClient::new(
            google_client_id,
            Some(google_client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:8855/oauth2callback".to_string())
                .expect("Invalid redirect URL"),
        );

        // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
        // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            // https://developers.google.com/identity/protocols/oauth2/scopes#logging
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/logging.read".to_string(),
            ))
            .add_extra_param("access_type", "offline") // needed to get a refresh token
            .set_pkce_challenge(pkce_challenge)
            .url();

        println!("Your browser has been opened to visit:\n\n\t{authorize_url}\n");

        webbrowser::open(authorize_url.as_str()).unwrap();

        let listener = TcpListener::bind("127.0.0.1:8855").unwrap();

        // Accept one connection
        let (mut stream, _) = listener.accept().unwrap();

        let code;
        let state;
        {
            let mut reader = BufReader::new(&stream);

            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();

            let redirect_url = request_line.split_whitespace().nth(1).unwrap();
            let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

            let code_pair = url
                .query_pairs()
                .find(|pair| {
                    let &(ref key, _) = pair;
                    key == "code"
                })
                .unwrap();

            let (_, value) = code_pair;
            code = AuthorizationCode::new(value.into_owned());

            let state_pair = url
                .query_pairs()
                .find(|pair| {
                    let &(ref key, _) = pair;
                    key == "state"
                })
                .unwrap();

            let (_, value) = state_pair;
            state = CsrfToken::new(value.into_owned());
        }

        let message = "Go back to your terminal :)";
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            message.len(),
            message
        );
        stream.write_all(response.as_bytes()).unwrap();

        println!("Google returned the following code:\n{}\n", code.secret());
        println!(
            "Google returned the following state:\n{} (expected `{}`)\n",
            state.secret(),
            csrf_state.secret()
        );

        // Exchange the code with a token.
        let token_response = client
            .exchange_code(code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .unwrap();

        println!(
            "Google returned the following token:\n{:?}\n",
            token_response
        );

        let refresh_token = token_response
            .refresh_token()
            .expect("Server did not return a refresh token");

        println!(
            "Google returned the following refresh token:\n{}\n",
            refresh_token.secret()
        );

        let access_token = token_response.access_token();
        println!(
            "Google returned the following access token:\n{}\n",
            access_token.secret()
        );
        let expires_in = token_response
            .expires_in()
            .expect("Server did not return access token expiration");

        let now = Utc::now();
        let expiry = now
            .checked_add_signed(Duration::from_std(expires_in).unwrap())
            .unwrap()
            .to_rfc3339();

        println!("Access token expires at: {}", expiry);
    }

    pub async fn get_logs() {
        use google_logging2::{
            oauth2::{ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod},
            Logging,
        };

        let secret = ApplicationSecret {
            client_id: env::var("GOOGLE_CLIENT_ID")
                .expect("Missing the GOOGLE_CLIENT_ID environment variable."),
            client_secret: env::var("GOOGLE_CLIENT_SECRET")
                .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
            redirect_uris: vec!["http://127.0.0.1:8855/oauth2callback".to_string()],
            project_id: Some("mv-prod-wukong-api".to_string()),
            client_email: None,
            auth_provider_x509_cert_url: Some(
                "https://www.googleapis.com/oauth2/v1/certs".to_string(),
            ),
            client_x509_cert_url: None,
        };

        let auth = InstalledFlowAuthenticator::builder(
            secret,
            InstalledFlowReturnMethod::HTTPPortRedirect(8855),
        )
        .build()
        .await
        .unwrap();

        // let scopes = &["https://www.googleapis.com/auth/logging.read"];
        // let token = auth.token(scopes).await.unwrap();
        //
        // println!("token {:#?}", token);

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

        let mut request = ListLogEntriesRequest::default();
        let mut call = hub.entries().list(request);
        let response = call
            .add_scope("https://www.googleapis.com/auth/logging.read")
            .doit()
            .await
            .unwrap();
        println!("response {:#?}", response);
    }
}
