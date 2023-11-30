use crate::{
    config::{AuthConfig, Config},
    error::{AuthError, WKCliError},
    utils::compare_with_current_time,
};
use aion::*;
use chrono::{DateTime, Duration, Utc};
use log::debug;
use openidconnect::{
    core::{
        CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata, CoreResponseType,
    },
    reqwest::async_http_client,
    AccessToken, AccessTokenHash, AdditionalClaims, AuthenticationFlow, AuthorizationCode,
    ClientId, CsrfToken, IntrospectionUrl, IssuerUrl, Nonce, OAuth2TokenResponse,
    PkceCodeChallenge, RedirectUrl, RefreshToken, Scope, TokenIntrospectionResponse,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};
use url::Url;

const EXPIRY_REMAINING_TIME_IN_MINS: i64 = 5;

#[derive(Debug, Deserialize, Serialize)]
struct OktaClaims;
impl AdditionalClaims for OktaClaims {}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub id_token: String,
    pub access_token: String,
    pub expiry_time: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenIntrospection {
    pub active: bool,
    pub exp: Option<DateTime<Utc>>,
    pub iat: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct OktaAuth {
    pub account: String,
    pub subject: String,
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expiry_time: String,
}

impl From<OktaAuth> for AuthConfig {
    fn from(value: OktaAuth) -> Self {
        Self {
            account: value.account,
            subject: value.subject,
            id_token: value.id_token,
            access_token: value.access_token,
            expiry_time: value.expiry_time,
            refresh_token: value.refresh_token,
        }
    }
}

pub fn need_tokens_refresh(config: &Config) -> Result<bool, WKCliError> {
    let auth_config = config.auth.as_ref().ok_or(WKCliError::UnAuthenticated)?;

    let remaining_duration = compare_with_current_time(&auth_config.expiry_time);
    Ok(remaining_duration < EXPIRY_REMAINING_TIME_IN_MINS.minutes())
}

pub async fn login(config: &Config) -> Result<OktaAuth, WKCliError> {
    let okta_client_id = ClientId::new(config.core.okta_client_id.clone());

    let issuer_url =
        IssuerUrl::new("https://mindvalley.okta.com".to_string()).expect("Invalid issuer URL");

    // Fetch Okta's OpenID Connect discovery document.
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .map_err(|_err| AuthError::OpenIDDiscoveryError)?;

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

    println!("Your browser has been opened to visit:\n\n\t{authorize_url}\n");

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
                let (key, _) = pair;
                key == "code"
            })
            .unwrap();

        let (_, value) = code_pair;
        code = AuthorizationCode::new(value.into_owned());

        let state_pair = url
            .query_pairs()
            .find(|pair| {
                let (key, _) = pair;
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
        .map_err(|err| match err {
            openidconnect::RequestTokenError::ServerResponse(error) => {
                let error_description = error.to_string();
                if error.error().to_string() == "invalid_grant"
                    && error_description.contains("refresh token")
                    && error_description.contains("expired")
                {
                    AuthError::OktaRefreshTokenExpired {
                        message: error_description,
                    }
                } else {
                    AuthError::OpenIDConnectError {
                        message: "Failed to contact token endpoint".to_string(),
                    }
                }
            }
            _ => AuthError::OpenIDConnectError {
                message: "Failed to contact token endpoint".to_string(),
            },
        })?;

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

    let id_token_claims: &CoreIdTokenClaims =
        id_token
            .claims(&id_token_verifier, &nonce)
            .map_err(|_err| AuthError::OpenIDConnectError {
                message: "Failed to verify ID token".to_string(),
            })?;

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

    Ok(OktaAuth {
        account: current_user_email.to_string(),
        subject: id_token_claims.subject().to_string(),
        id_token: id_token.to_string(),
        access_token: access_token.secret().to_owned(),
        expiry_time: expiry,
        refresh_token: refresh_token.secret().to_owned(),
    })
}

pub async fn refresh_tokens(config: &Config) -> Result<OktaAuth, WKCliError> {
    let auth_config = config.auth.as_ref().ok_or(WKCliError::UnAuthenticated)?;
    let okta_client_id = ClientId::new(config.core.okta_client_id.clone());

    let issuer_url =
        IssuerUrl::new("https://mindvalley.okta.com".to_string()).expect("Invalid issuer URL");

    // Fetch Okta's OpenID Connect discovery document.
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .map_err(|_err| AuthError::OpenIDDiscoveryError)?;

    // Set up the config for the Okta OAuth2 process.
    let client = CoreClient::from_provider_metadata(provider_metadata, okta_client_id, None);

    let token_exchange_result = client
        .exchange_refresh_token(&RefreshToken::new(auth_config.refresh_token.clone()))
        .request_async(async_http_client)
        .await;

    let token_response = match token_exchange_result {
        Ok(token_response) => token_response,
        Err(exchange_error) => {
            let error = match exchange_error {
                openidconnect::RequestTokenError::ServerResponse(error) => {
                    let error_description = error.to_string();
                    debug!("token_response: {:?}", error);

                    if error.error().to_string() == "invalid_grant"
                        && error_description.contains("refresh token")
                        && error_description.contains("expired")
                    {
                        introspect_token(config, &auth_config.refresh_token).await?;

                        AuthError::OktaRefreshTokenExpired {
                            message: error_description,
                        }
                    } else {
                        AuthError::OpenIDConnectError {
                            message: "Failed to contact token endpoint".to_string(),
                        }
                    }
                }
                _ => AuthError::OpenIDConnectError {
                    message: "Failed to contact token endpoint".to_string(),
                },
            };

            Err(error)?
        }
    };

    let id_token = token_response
        .extra_fields()
        .id_token()
        .expect("Server did not return an ID token");

    let new_refresh_token = token_response
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

    Ok(OktaAuth {
        account: auth_config.account.clone(),
        subject: auth_config.subject.clone(),
        id_token: id_token.to_string(),
        access_token: access_token.secret().to_owned(),
        expiry_time: expiry,
        refresh_token: new_refresh_token.secret().to_owned(),
    })
}

pub async fn introspect_token(
    config: &Config,
    token: &str,
) -> Result<TokenIntrospection, WKCliError> {
    let okta_client_id = ClientId::new(config.core.okta_client_id.clone());

    let issuer_url =
        IssuerUrl::new("https://mindvalley.okta.com".to_string()).expect("Invalid issuer URL");

    // Fetch Okta's OpenID Connect discovery document.
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .map_err(|_err| AuthError::OpenIDDiscoveryError)?;

    // Set up the config for the Okta OAuth2 process.
    let client = CoreClient::from_provider_metadata(provider_metadata, okta_client_id, None)
        .set_introspection_uri(
            IntrospectionUrl::new("https://mindvalley.okta.com/oauth2/v1/introspect".to_string())
                .expect("Invalid introspect URL"),
        );

    let token_response = client
        .introspect(&AccessToken::new(token.to_owned()))
        .map_err(|_err| AuthError::OpenIDConnectError {
            message: "Failed to contact introspect endpoint - 1".to_string(),
        })?
        .set_token_type_hint("refresh_token")
        .request_async(async_http_client)
        .await
        .map_err(|_err| AuthError::OpenIDConnectError {
            message: "Failed to get introspect response".to_string(),
        })?;

    debug!(
        "introspect response: refresh_token: {}, email: {:?}, active: {:?}, exp: {:?}",
        token,
        token_response.to_owned().username(),
        token_response.to_owned().active(),
        token_response.to_owned().exp()
    );

    Ok(TokenIntrospection {
        active: token_response.active(),
        exp: token_response.exp(),
        iat: token_response.iat(),
    })
}
