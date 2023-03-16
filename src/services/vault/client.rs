use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    pub client_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub auth: Auth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchLists {
    pub auth: Auth,
}

mod api_vault_url {
    pub const LOGIN: &str = "https://bunker.mindvalley.dev:8200/v1/auth/okta/login";
    pub const FETCH_LISTS: &str = "https://bunker.mindvalley.dev:8200/v1/secret/data";
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

    pub async fn fetch_lists(
        &self,
        api_token: &str,
        path: &str,
    ) -> Result<FetchLists, reqwest::Error> {
        let client = reqwest::Client::new();

        client
            .get(api_vault_url::FETCH_LISTS)
            .query(&[("path", path)])
            .header("X-Vault-Token", api_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}
