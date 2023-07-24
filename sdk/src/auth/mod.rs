use crate::error::AuthError;
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
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
struct OktaClaims;
impl AdditionalClaims for OktaClaims {}

#[derive(Debug)]
pub struct AuthInfo {
    pub account: String,
    pub subject: String,
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

pub struct Auth {
    api_key: String,
}

impl Auth {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}
