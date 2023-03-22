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
    base_url: String,
}

impl VaultClient {
    const LOGIN: &str = "/v1/auth/okta/login";
    const VERIFY_TOKEN: &str = "/v1/auth/token/lookup-self";
    const FETCH_SECRETS: &str = "/v1/secret/data";
    const UPDATE_SECRET: &str = "/v1/secret/data";

    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            base_url: "https://bunker.mindvalley.dev:8200".to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!(
            "{base_url}{base_path}/{email}",
            base_path = Self::LOGIN,
            email = email,
            base_url = self.base_url
        );

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
            "{base_url}{base_path}/{path}",
            base_url = self.base_url,
            base_path = Self::FETCH_SECRETS,
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
        let url = format!(
            "{base_url}{base_path}",
            base_url = self.base_url,
            base_path = Self::VERIFY_TOKEN,
        );

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
        key: &str,
        value: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!(
            "{base_url}{base_path}/{path}",
            base_url = self.base_url,
            base_path = Self::UPDATE_SECRET,
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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_login() {
        let server = MockServer::start();

        let email = "test@example.com";
        let password = "test_password";

        let api_resp = r#"
            {
              "auth": {
                "client_token": "test_token"
                }
            }"#;

        let mock_server = server.mock(|when, then| {
            when.method(POST)
                .path_contains(VaultClient::LOGIN)
                .body(format!("password={}", password));
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let vault_client = VaultClient::new().with_base_url(server.base_url());
        let response = vault_client.login(email, password).await;

        mock_server.assert();
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status(), 200);

        let login_data = response.json::<Login>().await.unwrap();
        assert_eq!(login_data.auth.client_token, "test_token");
    }

    #[tokio::test]
    async fn verify_token() {
        let server = MockServer::start();

        let api_token = "secret_token";

        let api_resp = r#"
            {
              "data": {
                "expire_time": "2019-12-10T10:10:10.000000Z",
                "issue_time": "2019-10-10T10:10:10.000000Z"
                }
            }"#;

        let mock_server = server.mock(|when, then| {
            when.method(GET)
                .path_contains(VaultClient::VERIFY_TOKEN)
                .header("X-Vault-Token", api_token);
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let vault_client = VaultClient::new().with_base_url(server.base_url());
        let response = vault_client.verify_token(api_token).await;

        mock_server.assert();
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status(), 200);

        let verify_token = response.json::<VerifyToken>().await.unwrap();

        assert!(
            !verify_token.data.expire_time.is_empty(),
            "Value should not be None"
        );
    }

    #[tokio::test]
    async fn fetch_secrets() {
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
    async fn update_secret() {
        let server = MockServer::start();

        let api_token = "test_token";
        let path = "devenv/test";
        let key = "update_key";
        let value = "updated_value";

        let api_resp = r#"
            {
              "data": {
                "test2": "secret_token"
                }
            }"#;

        let mock_server = server.mock(|when, then| {
            when.method(PUT)
                .path_contains(VaultClient::UPDATE_SECRET)
                .body(format!(r#"{{"data":{{"{}":"{}"}}}}"#, key, value))
                .header("X-Vault-Token", api_token);
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let vault_client = VaultClient::new().with_base_url(server.base_url());

        let response = vault_client
            .update_secret(api_token, path, key, value)
            .await;

        mock_server.assert();
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status(), 200);
    }
}
