use crate::{error::CliError, GlobalContext};

use super::DeploymentVersion;

pub async fn handle_execute<'a>(
    _context: GlobalContext,
    _namespace: &Option<String>,
    _artifact: &Option<String>,
    version: &Option<DeploymentVersion>,
) -> Result<bool, CliError<'a>> {
    todo!()
}
