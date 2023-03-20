use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    pub client_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub auth: Auth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchListData {
    pub data: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchLists {
    pub data: FetchListData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyTokenData {
    pub expire_time: String,
    pub issue_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyToken {
    pub data: VerifyTokenData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSecret {
    wrap_info: Option<String>,
    warnings: Option<String>,
    auth: Option<Auth>,
}

mod api_vault_url {
    pub const LOGIN: &str = "https://bunker.mindvalley.dev:8200/v1/auth/okta/login";
    pub const VERIFY_TOKEN: &str = "https://bunker.mindvalley.dev:8200/v1/auth/token/lookup-self";
    pub const FETCH_SECRETS: &str = "https://bunker.mindvalley.dev:8200/v1/secret/data";
    pub const PATCH_SECRET: &str = "https://bunker.mindvalley.dev:8200/v1/secret/data";
}

pub struct VaultClient {}

impl VaultClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<Login, reqwest::Error> {
        let url = format!(
            "{base_url}/{email}",
            base_url = api_vault_url::LOGIN,
            email = email
        );

        let client = reqwest::Client::new();

        client
            .post(url)
            .form(&[("password", password)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn fetch_secrets(
        &self,
        api_token: &str,
        path: &str,
    ) -> Result<FetchLists, reqwest::Error> {
        let url = format!(
            "{base_url}/{path}",
            base_url = api_vault_url::FETCH_SECRETS,
            path = path
        );

        let client = reqwest::Client::new();

        client
            .get(url)
            .header("X-Vault-Token", api_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn verify_token(&self, api_token: &str) -> Result<VerifyToken, reqwest::Error> {
        let client = reqwest::Client::new();

        client
            .get(api_vault_url::VERIFY_TOKEN)
            .header("X-Vault-Token", api_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn update_secret(
        &self,
        api_token: &str,
        path: &str,
        key: &str,
        value: &str,
    ) -> Result<UpdateSecret, reqwest::Error> {
        let url = format!(
            "{base_url}/{path}",
            base_url = api_vault_url::PATCH_SECRET,
            path = path
        );

        let mut secret_data = HashMap::new();
        let mut data = HashMap::new();
        data.insert(key.to_string(), value.to_string());
        secret_data.insert("data", data);

        let client = reqwest::Client::new();

        client
            .put(url)
            .header("X-Vault-Token", api_token)
            .json(&secret_data)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}
