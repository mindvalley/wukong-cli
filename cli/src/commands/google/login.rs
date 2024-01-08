use crate::{auth, error::WKCliError, loader::new_spinner};

pub async fn handle_login() -> Result<bool, WKCliError> {
    let loader = new_spinner();
    loader.set_message("Logging in to Google Cloud ...");

    auth::google_cloud::get_token_or_login().await;

    loader.finish_with_message(
        "Successfully logged in to Google Cloud. You can now use Wukong to manage your Google Cloud resources.\n",
    );

    Ok(true)
}
