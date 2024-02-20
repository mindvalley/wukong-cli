use crate::{auth, config::Config, error::WKCliError, loader::new_spinner};

pub async fn handle_login(config: Option<Config>) -> Result<bool, WKCliError> {
    let loader = new_spinner().with_message("Logging in to Google Cloud ...");
    auth::google_cloud::get_token_or_login(config).await;
    loader.finish_and_clear();

    println!("You are logged into Google Cloud. You can now use Wukong to manage your Google Cloud resources");

    Ok(true)
}
