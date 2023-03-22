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
pub struct FetchSecretsData {
    pub data: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchSecrets {
    pub data: FetchSecretsData,
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

pub struct VaultClient {
    client: reqwest::Client,
}

impl VaultClient {
    const LOGIN: &str = "https://bunker.mindvalley.dev:8200/v1/auth/okta/login";
    const VERIFY_TOKEN: &str = "https://bunker.mindvalley.dev:8200/v1/auth/token/lookup-self";
    const FETCH_SECRETS: &str = "https://bunker.mindvalley.dev:8200/v1/secret/data";
    const UPDATE_SECRET: &str = "https://bunker.mindvalley.dev:8200/v1/secret/data";

    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{base_url}/{email}", base_url = Self::LOGIN, email = email);

        let response = self
            .client
            .post(url)
            .form(&[("password", password)])
            .send()
            .await?;

        Ok(response)
    }

    pub async fn fetch_secrets(
        &self,
        api_token: &str,
        path: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!(
            "{base_url}/{path}",
            base_url = Self::FETCH_SECRETS,
            path = path
        );

        let response = self
            .client
            .get(url)
            .header("X-Vault-Token", api_token)
            .send()
            .await?;

        Ok(response)
    }

    pub async fn verify_token(&self, api_token: &str) -> Result<reqwest::Response, reqwest::Error> {
        let response = self
            .client
            .get(Self::VERIFY_TOKEN)
            .header("X-Vault-Token", api_token)
            .send()
            .await?;

        Ok(response)
    }

    pub async fn update_secret(
        &self,
        api_token: &str,
        path: &str,
        key: &str,
        value: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!(
            "{base_url}/{path}",
            base_url = Self::UPDATE_SECRET,
            path = path
        );

        let mut secret_data = HashMap::new();
        let mut data = HashMap::new();
        data.insert(key.to_string(), value.to_string());
        secret_data.insert("data", data);

        let response = self
            .client
            .put(url)
            .header("X-Vault-Token", api_token)
            .json(&secret_data)
            .send()
            .await?;

        Ok(response)
    }
}
