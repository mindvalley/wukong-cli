use crate::{
    config::{AuthConfig, CONFIG_FILE},
    Config as CLIConfig,
};
use chrono::{Duration, Utc};
use openidconnect::{
    core::{
        CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata, CoreResponseType,
    },
    reqwest::async_http_client,
    AccessTokenHash, AdditionalClaims, AuthenticationFlow, AuthorizationCode, ClientId, CsrfToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl, Scope, UserInfoClaims,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    process::exit,
};
use url::Url;
use webbrowser;

#[derive(Debug, Deserialize, Serialize)]
struct OktaClaims;
impl AdditionalClaims for OktaClaims {}

fn handle_error<T: std::error::Error>(fail: &T, msg: &'static str) {
    let mut err_msg = format!("ERROR: {}", msg);
    let mut cur_fail: Option<&dyn std::error::Error> = Some(fail);
    while let Some(cause) = cur_fail {
        err_msg += &format!("\n    caused by: {}", cause);
        cur_fail = cause.source();
    }
    println!("{}", err_msg);
    exit(1);
}

pub async fn login() {
    let okta_client_id = ClientId::new("0oakfxaegyAV5JDD5357".to_string());

    let issuer_url =
        IssuerUrl::new("https://mindvalley.okta.com".to_string()).expect("Invalid issuer URL");

    // Fetch Okta's OpenID Connect discovery document.
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to discover OpenID Provider");
            unreachable!();
        });

    // Set up the config for the Okta OAuth2 process.
    let client = CoreClient::from_provider_metadata(provider_metadata, okta_client_id, None)
        // This example will be running its own server at localhost:8080.
        // See below for the server implementation.
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:6758/login/callback".to_string())
                .expect("Invalid redirect URL"),
        );

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // This example is requesting access to the the user's profile including email.
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

    webbrowser::open(&authorize_url.to_string()).unwrap();

    let listener = TcpListener::bind("127.0.0.1:6758").unwrap();

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

    let message = "You are now authenticated with the wukong CLI! The authentication flow has completed successfully. You can close this window and go back to your terminal :)";
    let response = format!(
        "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
        message.len(),
        message
    );
    stream.write_all(response.as_bytes()).unwrap();

    // println!("Okta returned the following code:\n{}\n", code.secret());
    // println!(
    //     "Okta returned the following state:\n{} (expected `{}`)\n",
    //     state.secret(),
    //     csrf_state.secret()
    // );

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

    // println!(
    //     "Okta returned access token:\n{}\n",
    //     token_response.access_token().secret()
    // );
    // println!("Okta returned scopes: {:?}", token_response.scopes());

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
    // println!("Okta returned ID token: {:?}\n", id_token_claims);
    // println!("Okta returned refresh token: {:?}", refresh_token.secret());
    // println!("Okta returned access token: {:?}", access_token.secret());
    // println!("Okta returned access token expiration: {:?}", expires_in);

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

    // The authenticated user's identity is now available. See the IdTokenClaims struct for a
    // complete listing of the available claims.
    // println!(
    //     "User {} with e-mail address {} has authenticated successfully",
    //     id_token_claims.subject().as_str(),
    //     id_token_claims
    //         .email()
    //         .map(|email| email.as_str())
    //         .unwrap_or("<not provided>"),
    // );

    let current_user_email = id_token_claims
        .email()
        .map(|email| email.as_str())
        .unwrap_or("<email not provided>");

    let config_file = CONFIG_FILE
        .as_ref()
        .expect("Unable to identify user's home directory");

    let now = Utc::now();
    let expiry = now
        .checked_add_signed(Duration::from_std(expires_in).unwrap())
        .unwrap()
        .to_rfc3339();
    // println!("expiry: {:?}", expiry);

    match CLIConfig::load(&config_file) {
        Ok(mut config) => {
            config.auth = Some(AuthConfig {
                current_user: current_user_email.to_string(),
                access_token: access_token.secret().to_owned(),
                expiry_time: expiry.to_string(),
                refresh_token: refresh_token.secret().to_owned(),
            });
            // config.core.application = config_value.to_string();
            config.save(&config_file).unwrap();
            println!("You are now logged in as [{}].", current_user_email);
        }
        Err(_err) => todo!(),
    };

    // let userinfo_claims: UserInfoClaims<OktaClaims, CoreGenderClaim> = client
    //     .user_info(token_response.access_token().to_owned(), None)
    //     .unwrap_or_else(|err| {
    //         handle_error(&err, "No user info endpoint");
    //         unreachable!();
    //     })
    //     .request_async(async_http_client)
    //     .await
    //     .unwrap_or_else(|err| {
    //         println!("err: {:?}", err);
    //         handle_error(&err, "Failed requesting user info");
    //         unreachable!();
    //     });
    // println!("Okta returned UserInfo: {:?}", userinfo_claims);
}
