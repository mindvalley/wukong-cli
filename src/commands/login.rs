use crate::{auth, error::CliError, GlobalContext};

pub async fn handle_login<'a>(_context: GlobalContext) -> Result<bool, CliError<'a>> {
    auth::login().await;
    Ok(true)
}
