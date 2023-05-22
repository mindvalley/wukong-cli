use crate::{error::CliError, graphql::QueryClient};

pub async fn handle_connect(
    client: &QueryClient,
    name: &str,
    port: &str,
) -> Result<bool, CliError> {
    let (namespace, version, instance_name) = parse_name(name)?;

    Ok(true)
}

fn parse_name(name: &str) -> Result<(String, String, String), CliError> {
    if let Some((instance_info, instance_name)) = name.split_once("/") {
        if let Some((namespace, version)) = instance_info.split_once("@") {
            return Ok((
                namespace.to_string(),
                version.to_string(),
                instance_name.to_string(),
            ));
        }
    }

    todo!()
}
