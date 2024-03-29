use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    pub client_token: String,
    pub lease_duration: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Renew {
    pub auth: Auth,
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
    base_url: String,
}

impl Default for VaultClient {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultClient {
    pub const FETCH_SECRETS: &'static str = "/v1/secret/data";
    pub const UPDATE_SECRET: &'static str = "/v1/secret/data";

    pub fn new() -> Self {
        let client = reqwest::Client::new();
        #[cfg(feature = "prod")]
        let base_url = "https://bunker.mindvalley.dev:8200".to_string();

        #[cfg(not(feature = "prod"))]
        let base_url = match std::env::var("WUKONG_DEV_VAULT_API_URL") {
            Ok(vault_api_url) => vault_api_url,
            Err(_) => "https://bunker.mindvalley.dev:8200".to_string(),
        };

        Self { client, base_url }
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub async fn fetch_secrets(
        &self,
        api_token: &str,
        path: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}/{}", self.base_url, Self::FETCH_SECRETS, path);

        let response = self
            .client
            .get(url)
            .header("X-Vault-Token", api_token)
            .send()
            .await?;

        Ok(response)
    }

    pub async fn update_secret(
        &self,
        api_token: &str,
        path: &str,
        data: &HashMap<&str, &str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}/{}", self.base_url, Self::UPDATE_SECRET, path);

        // Update the secret with updated value:
        let mut secret_data = HashMap::new();
        secret_data.insert("data", data);

        let response = self
            .client
            .patch(url)
            .header("X-Vault-Token", api_token)
            .header("Content-Type", "application/merge-patch+json")
            .json(&secret_data)
            .send()
            .await?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_fetch_secrets() {
        let server = MockServer::start();

        let api_token = "test_token";
        let path = "devenv/test";

        let api_resp = r#"
            {
              "data": {
                "data": {
                    "test2": "secret_token"
                    }
                }
            }"#;

        let mock_server = server.mock(|when, then| {
            when.method(GET)
                .path_contains("/v1/secret/data")
                .path_contains(path)
                .header("X-Vault-Token", api_token);
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let vault_client = VaultClient::new().with_base_url(server.base_url());
        let response = vault_client.fetch_secrets(api_token, path).await;

        mock_server.assert();
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status(), 200);

        let secrets = response.json::<FetchSecrets>().await.unwrap();

        let data = secrets.data.data.get("test2").unwrap();
        assert_eq!(data, "secret_token");
    }

    #[tokio::test]
    async fn test_update_secret() {
        let server = MockServer::start();

        let api_token = "test_token";
        let path = "devenv/test";
        let mut update_data = HashMap::new();
        update_data.insert("test", "test4");

        let api_resp = r#"
            {
              "data": {
                "test": "test4"
                }
            }"#;

        let mock_server = server.mock(|when, then| {
            when.method("PATCH")
                .path_contains(VaultClient::UPDATE_SECRET)
                .body(format!(r#"{{"data":{{"{}":"{}"}}}}"#, "test", "test4"))
                .header("X-Vault-Token", api_token);
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let vault_client = VaultClient::new().with_base_url(server.base_url());

        let response = vault_client
            .update_secret(api_token, path, &update_data)
            .await;

        mock_server.assert();
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status(), 200);
    }
}
