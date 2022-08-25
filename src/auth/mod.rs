use crate::{error::CliError, SETTINGS};
use chrono::{Duration, Utc};
use openidconnect::{
    core::{
        CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata, CoreResponseType,
    },
    reqwest::async_http_client,
    AccessTokenHash, AdditionalClaims, AuthenticationFlow, AuthorizationCode, ClientId, CsrfToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl, RefreshToken, Scope,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::Write as _,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    process::exit,
};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
struct OktaClaims;
impl AdditionalClaims for OktaClaims {}

#[derive(Debug)]
pub struct AuthInfo {
    pub account: String,
    pub id_token: String,
    pub access_token: String,
    pub expiry_time: String,
    pub refresh_token: String,
}

#[derive(Debug)]
pub struct TokenInfo {
    pub id_token: String,
    pub access_token: String,
    pub expiry_time: String,
    pub refresh_token: String,
}

fn handle_error(fail: &impl Error, msg: &'static str) {
    let mut err_msg = format!("ERROR: {}", msg);
    let mut cur_fail: Option<&dyn Error> = Some(fail);
    while let Some(cause) = cur_fail {
        write!(err_msg, "\n    caused by: {}", cause).unwrap();
        cur_fail = cause.source();
    }
    println!("{}", err_msg);
    exit(1);
}

pub async fn login<'a>() -> Result<AuthInfo, CliError<'a>> {
    let okta_client_id = ClientId::new(SETTINGS.okta.client_id.clone());

    let issuer_url =
        IssuerUrl::new("https://mindvalley.okta.com".to_string()).expect("Invalid issuer URL");

    // Fetch Okta's OpenID Connect discovery document.
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .map_err(|_err| CliError::OpenIDDiscoveryError)?;

    // Set up the config for the Okta OAuth2 process.
    let client = CoreClient::from_provider_metadata(provider_metadata, okta_client_id, None)
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:6758/login/callback".to_string())
                .expect("Invalid redirect URL"),
        );

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, _csrf_state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!(
        "Your browser has been opened to visit:\n\n\t{}\n",
        authorize_url
    );

    webbrowser::open(authorize_url.as_str()).unwrap();

    let listener = TcpListener::bind("127.0.0.1:6758").unwrap();

    // Accept one connection
    let (mut stream, _) = listener.accept().unwrap();
    let code;
    // let state;
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

        let (_, _value) = state_pair;
        // state = CsrfToken::new(value.into_owned());
    }

    let message = "You are now authenticated with the wukong CLI! The authentication flow has completed successfully. You can close this window and go back to your terminal :)";
    let response = format!(
        "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
        message.len(),
        message
    );
    stream.write_all(response.as_bytes()).unwrap();

    // Exchange the code with a token.
    let token_response = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .unwrap_or_else(|err| {
            println!("{:?}", err);
            handle_error(&err, "Failed to contact token endpoint");
            unreachable!();
        });

    let id_token_verifier: CoreIdTokenVerifier = client.id_token_verifier();
    let id_token = token_response
        .extra_fields()
        .id_token()
        .expect("Server did not return an ID token");

    let refresh_token = token_response
        .refresh_token()
        .expect("Server did not return a refresh token");

    // println!("resp: {:?}", token_response);
    let access_token = token_response.access_token();

    let expires_in = token_response
        .expires_in()
        .expect("Server did not return access token expiration");

    let id_token_claims: &CoreIdTokenClaims = id_token
        .claims(&id_token_verifier, &nonce)
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to verify ID token");
            unreachable!();
        });

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
    if let Some(expected_access_token_hash) = id_token_claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            &id_token.signing_alg().unwrap(),
        )
        .unwrap();
        if actual_access_token_hash != *expected_access_token_hash {
            panic!("Invalid access token");
        }
    }

    let current_user_email = id_token_claims
        .email()
        .map(|email| email.as_str())
        .unwrap_or("<email not provided>");

    let now = Utc::now();
    let expiry = now
        .checked_add_signed(Duration::from_std(expires_in).unwrap())
        .unwrap()
        .to_rfc3339();

    Ok(AuthInfo {
        account: current_user_email.to_string(),
        id_token: id_token.to_string(),
        access_token: access_token.secret().to_owned(),
        expiry_time: expiry,
        refresh_token: refresh_token.secret().to_owned(),
    })
}

pub async fn refresh_tokens(refresh_token: &RefreshToken) -> Result<TokenInfo, CliError> {
    let okta_client_id = ClientId::new(SETTINGS.okta.client_id.clone());

    let issuer_url =
        IssuerUrl::new("https://mindvalley.okta.com".to_string()).expect("Invalid issuer URL");

    // Fetch Okta's OpenID Connect discovery document.
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .map_err(|_err| CliError::OpenIDDiscoveryError)?;

    // Set up the config for the Okta OAuth2 process.
    let client = CoreClient::from_provider_metadata(provider_metadata, okta_client_id, None)
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:6758/login/callback".to_string())
                .expect("Invalid redirect URL"),
        );

    let token_response = client
        .exchange_refresh_token(refresh_token)
        .request_async(async_http_client)
        .await
        .unwrap_or_else(|err| {
            println!("{:?}", err);
            handle_error(&err, "Failed to contact token endpoint");
            unreachable!();
        });

    let id_token = token_response
        .extra_fields()
        .id_token()
        .expect("Server did not return an ID token");

    let refresh_token = token_response
        .refresh_token()
        .expect("Server did not return a refresh token");

    let access_token = token_response.access_token();

    let expires_in = token_response
        .expires_in()
        .expect("Server did not return access token expiration");

    let now = Utc::now();
    let expiry = now
        .checked_add_signed(Duration::from_std(expires_in).unwrap())
        .unwrap()
        .to_rfc3339();

    Ok(TokenInfo {
        id_token: id_token.to_string(),
        access_token: access_token.secret().to_owned(),
        expiry_time: expiry,
        refresh_token: refresh_token.secret().to_owned(),
    })
}
