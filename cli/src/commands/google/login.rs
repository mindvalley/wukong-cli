use crate::{auth, config::Config, error::WKCliError};

pub async fn handle_login(config: Option<Config>) -> Result<bool, WKCliError> {
    auth::google_cloud::get_token_or_login(config).await;
    println!("You are logged into Google Cloud. You can now use Wukong to manage your Google Cloud resources");
    Ok(true)
}
