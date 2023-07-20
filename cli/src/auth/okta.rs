use crate::{
    config::{AuthConfig, Config},
    error::WKCliError,
    utils::compare_with_current_time,
};
use aion::*;
use wukong_sdk::{OktaAuthResponse, OktaAuthenticator};

const EXPIRY_REMAINING_TIME_IN_MINS: i64 = 5;

pub struct OktaAuth {
    pub account: String,
    pub subject: String,
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expiry_time: String,
}

impl From<OktaAuthResponse> for OktaAuth {
    fn from(value: OktaAuthResponse) -> Self {
        Self {
            account: value.account,
            subject: value.subject,
            id_token: value.id_token,
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            expiry_time: value.expiry_time,
        }
    }
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

pub async fn login(config: &Config) -> Result<OktaAuth, WKCliError> {
    let okta_authenticator = OktaAuthenticator::builder()
        .with_okta_id(&config.core.okta_client_id)
        .with_callback_url("http://localhost:6758/login/callback")
        .build();

    let auth_resp = okta_authenticator.login().await?;
    Ok(auth_resp.into())
}

pub async fn refresh_tokens(config: &Config) -> Result<OktaAuth, WKCliError> {
    let auth_config = config.auth.as_ref().ok_or(WKCliError::UnAuthenticated)?;

    let okta_authenticator = OktaAuthenticator::builder()
        .with_okta_id(&config.core.okta_client_id)
        .with_callback_url("http://localhost:6758/login/callback")
        .build();

    let new_tokens = okta_authenticator
        .refresh_tokens(auth_config.refresh_token.clone())
        .await
        .map_err(|err| match err {
            wukong_sdk::error::AuthError::RefreshTokenExpired { message: _ } => {
                WKCliError::RefreshTokenExpired
            }
            _ => err.into(),
        })?;

    Ok(OktaAuth {
        account: auth_config.account.clone(),
        subject: auth_config.subject.clone(),
        id_token: new_tokens.id_token,
        access_token: new_tokens.access_token,
        refresh_token: new_tokens.refresh_token,
        expiry_time: new_tokens.expiry_time,
    })
}

pub fn need_tokens_refresh(config: &Config) -> Result<bool, WKCliError> {
    let auth_config = config.auth.as_ref().ok_or(WKCliError::UnAuthenticated)?;

    let remaining_duration = compare_with_current_time(&auth_config.expiry_time);
    Ok(remaining_duration < EXPIRY_REMAINING_TIME_IN_MINS.minutes())
}
