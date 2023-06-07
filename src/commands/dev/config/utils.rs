use crate::{
    error::{CliError, DevConfigError},
    services::vault::Vault,
    utils::secret_extractors::{
        ElixirConfigExtractor, SecretExtractor, SecretInfo, WKTomlConfigExtractor,
    },
};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use log::debug;
use owo_colors::OwoColorize;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use super::diff::has_diff;

pub async fn get_updated_configs<'a>(
    vault: &Vault,
    vault_token: &str,
    config_files: &'a Vec<(String, Vec<SecretInfo>)>,
) -> Result<Vec<(&'a SecretInfo, String, String, String)>, CliError> {
    // Comparing local vs remote ....
    println!("{}", "comparing local config vs remote config...".cyan());

    let mut updated_configs = Vec::new();

    for config_file in config_files {
        let (config_path, secret_infos) = config_file;
        for info in secret_infos {
            let remote_secrets = vault.get_secrets(vault_token, &info.src).await?.data;

            let local_config = get_local_config_as_string(&info.destination_file, config_path)
                .map_err(|error| {
                    debug!("Error: {:?}", error);
                    CliError::DevConfigError(DevConfigError::ConfigSecretNotFound)
                })?;

            // Get only one key from hashmap
            let remote_config = match remote_secrets.get(&info.name) {
                Some(config) => config,
                None => {
                    // return Err(CliError::DevConfigError(
                    //     DevConfigError::InvalidSecretPath {
                    //         config_path: make_path_relative(config_path),
                    //         annotation: format!(
                    //             "{}:{}/{}#{}",
                    //             vault_secret_annotation.source,
                    //             vault_secret_annotation.engine,
                    //             vault_secret_annotation.secret_path.clone(),
                    //             vault_secret_annotation.secret_name
                    //         ),
                    //     },
                    // ));
                    todo!()
                }
            };

            if has_diff(remote_config, &local_config) {
                updated_configs.push((
                    info,
                    remote_config.clone(),
                    local_config,
                    config_path.clone(),
                ));
            }
        }
    }

    Ok(updated_configs)
}

pub fn get_local_config_path(destination_file: &str, config_path: &str) -> PathBuf {
    let path = PathBuf::from(config_path);
    path.parent().unwrap().join(destination_file)
}

pub fn get_local_config_as_string(
    destination_file: &str,
    config_path: &str,
) -> Result<String, CliError> {
    let local_config_path = get_local_config_path(destination_file, config_path);
    let local_config = std::fs::read_to_string(local_config_path)?;

    Ok(local_config)
}

pub fn get_secret_config_files(current_path: Option<PathBuf>) -> Result<Vec<PathBuf>, CliError> {
    let current_path = current_path.unwrap_or(current_dir()?);

    let mut overrides = OverrideBuilder::new(current_path.clone());
    overrides.add("**/config/dev.exs").unwrap();
    overrides.add("**/.wukong.toml").unwrap();

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

pub fn extract_secret_infos(secret_config_files: Vec<PathBuf>) -> Vec<(String, Vec<SecretInfo>)> {
    let mut extracted_infos = Vec::new();

    for file in secret_config_files {
        match file.to_string_lossy() {
            x if x.contains(".wukong.toml") => {
                extracted_infos.push((
                    file.to_string_lossy().to_string(),
                    WKTomlConfigExtractor::extract(&file),
                ));
            }
            x if x.contains("dev.exs") => {
                extracted_infos.push((
                    file.to_string_lossy().to_string(),
                    ElixirConfigExtractor::extract(&file),
                ));
            }
            x => {
                debug!("Ignoring file: {}", x);
            }
        }
    }

    extracted_infos
}

// Test:
#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;

    use super::*;

    #[test]
    fn test_filter_config_with_secret_annotations() -> Result<(), Box<dyn std::error::Error>> {
        let dir = assert_fs::TempDir::new().unwrap();
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

        let has_file1_path = filtered_annotations
            .iter()
            .any(|(path, _)| path == &file1_path.to_string_lossy().to_string());

        let does_not_have_file2_path = filtered_annotations
            .iter()
            .any(|(path, _)| path == &file2_path.to_string_lossy().to_string());

        assert!(has_file1_path);
        assert!(!does_not_have_file2_path);

        let (_, annotation) = filtered_annotations
            .iter()
            .find(|(path, _)| path == &file1_path.to_string_lossy().to_string())
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
        let dir = assert_fs::TempDir::new().unwrap();
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
}
