use super::{ApplicationNamespace, ApplicationVersion};
use crate::{commands::Context, error::CliError};

pub async fn handle_logs(
    context: Context,
    namespace: &ApplicationNamespace,
    version: &ApplicationVersion,
    show_error_and_above: &bool,
    since: &Option<String>,
    until: &Option<String>,
    limit: &Option<String>,
) -> Result<bool, CliError> {
    Ok(true)
}
