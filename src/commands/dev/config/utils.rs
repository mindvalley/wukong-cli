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
                        config_path: make_path_relative(config_path),
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

pub fn get_local_config_as_string(
    destination_file: &str,
    config_path: &str,
) -> Result<String, CliError> {
    let path = PathBuf::from(config_path);
    let dir_path = path.parent().unwrap();
    let local_secrets = dir_path.join(destination_file);
    let local_secrets = std::fs::read_to_string(local_secrets)?;

    Ok(local_secrets)
}

pub fn get_dev_config_files() -> Result<Vec<PathBuf>, CliError> {
    let current_path = current_dir()?;

    let mut overrides = OverrideBuilder::new(current_path.clone());
    overrides.add("**/config/dev.exs").unwrap();

    let config_files = WalkBuilder::new(current_path)
        .overrides(overrides.build().unwrap())
        .build()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(config_files)
}

pub fn make_path_relative(path: &str) -> String {
    let current_dir = current_dir().unwrap();
    let path = Path::new(path);

    path.strip_prefix(current_dir)
        .map(|p| p.to_owned())
        .unwrap_or(path.to_owned())
        .into_os_string()
        .into_string()
        .unwrap()
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

// Test:
#[cfg(test)]
mod test {
    use std::env::set_current_dir;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_filter_config_with_secret_annotations() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let file1_path = dir.path().join("dev.exs");
        let file2_path = dir.path().join("dev2.exs");

        let mut file1 = File::create(&file1_path)?;
        writeln!(
            file1,
            r#"# Import development secrets
            # wukong.mindvalley.dev/config-secrets-location: vault:secret/wukong-cli/sandboxes#dev.secrets.exs
            import_config("dev.secrets.exs")"#
        )?;

        let mut file2 = File::create(&file2_path)?;
        writeln!(file2, "Some other content")?;

        let available_files = vec![file1_path.clone(), file2_path.clone()];
        let filtered_annotations = filter_config_with_secret_annotations(available_files)?;

        assert_eq!(filtered_annotations.len(), 1);
        assert!(filtered_annotations.contains_key(&file1_path.to_string_lossy().to_string()));

        let annotation = filtered_annotations
            .get(&file1_path.to_string_lossy().to_string())
            .unwrap();
        assert_eq!(
            annotation.key,
            "wukong.mindvalley.dev/config-secrets-location"
        );
        assert_eq!(annotation.source, "vault");
        assert_eq!(annotation.engine, "secret");

        dir.close()?;

        Ok(())
    }

    #[test]
    fn test_get_local_config_as_string() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let subdir = dir.path().join("config");
        std::fs::create_dir_all(&subdir)?;

        let file_path = subdir.join("config.exs");
        let config_content = "Some test content";

        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", config_content)?;

        let destination_file = "config/config.exs";
        let config_path = subdir.to_str().unwrap();

        let local_config = get_local_config_as_string(destination_file, config_path)?;

        assert_eq!(local_config.trim(), config_content);

        dir.close()?;

        Ok(())
    }

    #[test]
    fn test_make_path_relative() {
        let current_dir = current_dir().unwrap();
        let sub_path = PathBuf::from("/parent/some/config/dev.exs");
        let absolute_path = current_dir.join(&sub_path);

        let relative_path = make_path_relative(absolute_path.to_str().unwrap());

        // Use contains instead of equals because the path is relative to the current directory
        assert!(relative_path.contains("parent/some/config/dev.exs"));
    }

    #[test]
    fn test_get_dev_config_files() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let subdir1 = dir.path().join("config");
        let subdir2 = dir.path().join("another_config");
        create_dir_all(&subdir1)?;
        create_dir_all(&subdir2)?;

        let dev_config1 = subdir1.join("dev.exs");
        let non_target_file = subdir1.join("dev.secrets.exe");

        File::create(&dev_config1)?;
        File::create(&non_target_file)?;

        // Set the current directory to the temporary directory
        set_current_dir(dir.path())?;

        let config_files = get_dev_config_files()?;

        // Reset the current directory to the original one after the test
        set_current_dir(current_dir()?)?;

        // Get first item from the vector
        let expected_config_files = config_files.get(0).unwrap().to_str().unwrap();
        let expected_dev_files = dev_config1.to_str().unwrap();

        assert!(expected_config_files.contains(&expected_dev_files));

        dir.close()?;

        Ok(())
    }
}
