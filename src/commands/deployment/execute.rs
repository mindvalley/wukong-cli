use crate::{error::CliError, GlobalContext};

use super::DeploymentVersion;

pub async fn handle_execute<'a>(
    _context: GlobalContext,
    _namespace: &Option<String>,
    _version: &DeploymentVersion,
    _artifact: &Option<String>,
) -> Result<bool, CliError<'a>> {
    todo!()
}
