use crate::{
    error::{CliError, DevConfigError},
    services::vault::Vault,
    utils::annotations::{read_vault_annotation, VaultSecretAnnotation},
};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use log::debug;
use owo_colors::OwoColorize;
use std::{
    collections::HashMap,
    env::current_dir,
    path::{Path, PathBuf},
};

use super::diff::has_diff;

pub fn remove_parent_directories(path: &str) -> String {
    let current_dir = current_dir().unwrap();
    let path = Path::new(path);

    path.strip_prefix(current_dir)
        .map(|p| p.to_owned())
        .unwrap_or(path.to_owned())
        .into_os_string()
        .into_string()
        .unwrap()
}

pub async fn get_updated_configs(
    vault: &Vault,
    vault_token: &str,
    config_files: &HashMap<String, VaultSecretAnnotation>,
) -> Result<HashMap<String, (VaultSecretAnnotation, String, String)>, CliError> {
    // Comparing local vs remote ....
    println!("{}", "comparing local config vs remote config...".cyan());

    let mut updated_configs = HashMap::new();

    for config_file in config_files {
        let (config_path, vault_secret_annotation) = config_file;
        let remote_secrets = vault
            .get_secrets(vault_token, &vault_secret_annotation.secret_path)
            .await?
            .data;

        let local_config =
            get_local_config_as_string(&vault_secret_annotation.destination_file, config_path)
                .map_err(|error| {
                    debug!("Error: {:?}", error);
                    CliError::DevConfigError(DevConfigError::ConfigSecretNotFound)
                })?;

        // Get only one key from hashmap
        let remote_config = match remote_secrets.get(&vault_secret_annotation.secret_name) {
            Some(config) => config,
            None => {
                return Err(CliError::DevConfigError(
                    DevConfigError::InvalidSecretPath {
                        config_path: remove_parent_directories(config_path),
                    },
                ));
            }
        };

        if has_diff(remote_config, &local_config) {
            updated_configs.insert(
                config_path.clone(),
                (
                    vault_secret_annotation.clone(),
                    remote_config.clone(),
                    local_config,
                ),
            );
        }
    }

    Ok(updated_configs)
}

fn get_local_config_as_string(
    destination_file: &str,
    config_path: &str,
) -> Result<String, CliError> {
    let path = PathBuf::from(config_path);
    let dir_path = path.parent().unwrap();
    let local_secrets = dir_path.join(destination_file);
    let local_secrets = std::fs::read_to_string(local_secrets)?;

    Ok(local_secrets)
}

pub fn get_dev_config_files(path: &Path) -> Vec<PathBuf> {
    let mut overrides = OverrideBuilder::new(path);
    overrides.add("**/config/dev.exs").unwrap();

    WalkBuilder::new(path)
        .overrides(overrides.build().unwrap())
        .build()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn get_available_files() -> Result<Vec<PathBuf>, CliError> {
    let current_path = current_dir()?;
    let available_files = get_dev_config_files(&current_path);

    Ok(available_files)
}

pub fn filter_config_with_secret_annotations(
    available_files: Vec<PathBuf>,
) -> Result<HashMap<String, VaultSecretAnnotation>, CliError> {
    let mut filtered_annotations: HashMap<String, VaultSecretAnnotation> = HashMap::new();

    for file in available_files {
        let file_contents = std::fs::read_to_string(file.clone())?;
        let annotations = read_vault_annotation(&file_contents);

        for annotation in annotations {
            if annotation.key == "wukong.mindvalley.dev/config-secrets-location"
                && annotation.source == "vault"
                && annotation.engine == "secret"
            {
                filtered_annotations.insert(file.to_string_lossy().to_string(), annotation);
            }
        }
    }

    Ok(filtered_annotations)
}
