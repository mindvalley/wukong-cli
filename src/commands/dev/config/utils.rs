use crate::{
    error::{CliError, DevConfigError, ExtractError},
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

            let local_config = match get_local_config_as_string(&info.destination_file, config_path)
            {
                Ok(config) => config,
                Err(error) => {
                    debug!("Error: {:?}", error);

                    eprintln!(
                        "'{}' not found.",
                        make_path_relative(
                            &get_local_config_path(&info.destination_file, config_path)
                                .to_string_lossy()
                        )
                        .cyan()
                    );
                    continue;
                }
            };

            // Get only one key from hashmap
            let remote_config = match remote_secrets.get(&info.name) {
                Some(config) => config,
                None => {
                    return Err(CliError::DevConfigError(
                        DevConfigError::InvalidSecretPath {
                            config_path: make_path_relative(config_path),
                            annotation: info.key.to_string(),
                        },
                    ));
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

pub fn extract_secret_infos(
    secret_config_files: Vec<PathBuf>,
) -> Result<Vec<(String, Vec<SecretInfo>)>, ExtractError> {
    let mut extracted_infos = Vec::new();

    for file in secret_config_files {
        match file.to_string_lossy() {
            x if x.contains(".wukong.toml") => {
                extracted_infos.push((
                    file.to_string_lossy().to_string(),
                    WKTomlConfigExtractor::extract(&file)?,
                ));
            }
            x if x.contains("dev.exs") => {
                extracted_infos.push((
                    file.to_string_lossy().to_string(),
                    ElixirConfigExtractor::extract(&file)?,
                ));
            }
            x => {
                debug!("Ignoring file: {}", x);
            }
        }
    }

    Ok(extracted_infos)
}

// Test:
#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;

    use assert_fs::prelude::{FileTouch, PathChild};

    use super::*;

    #[test]
    fn test_secret_config_files() {
        // four files:
        // temp/config/dev.exs
        // temp/config/prod.exs
        // temp/app/config/dev.exs
        // temp/.wukong.toml

        let temp = assert_fs::TempDir::new().unwrap();
        let dev_config_file = temp.child("config/dev.exs");
        dev_config_file.touch().unwrap();
        let prod_config_file = temp.child("config/prod.exs");
        prod_config_file.touch().unwrap();
        let another_dev_config_file = temp.child("app/config/dev.exs");
        another_dev_config_file.touch().unwrap();
        let wukong_toml_file = temp.child(".wukong.toml");
        wukong_toml_file.touch().unwrap();

        let files = get_secret_config_files(Some(temp.to_path_buf())).unwrap();
        let files_names = files
            .iter()
            .map(|f| f.to_string_lossy())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 3);
        assert!(files_names.contains(&dev_config_file.path().to_string_lossy()));
        assert!(files_names.contains(&another_dev_config_file.path().to_string_lossy()));
        assert!(files_names.contains(&wukong_toml_file.path().to_string_lossy()));

        temp.close().unwrap();
    }

    #[test]
    fn test_extract_secret_infos() -> Result<(), Box<dyn std::error::Error>> {
        let dir = assert_fs::TempDir::new().unwrap();
        let file1_path = dir.path().join("dev.exs");
        let file2_path = dir.path().join("dev2.exs");
        let file3_path = dir.path().join(".wukong.toml");

        let mut file1 = File::create(&file1_path)?;
        writeln!(
            file1,
            r#"# Import development secrets
            # wukong.mindvalley.dev/config-secrets-location: vault:secret/wukong-cli/sandboxes#dev.secrets.exs
            import_config("dev.secrets.exs")"#
        )?;

        let mut file2 = File::create(&file2_path)?;
        writeln!(file2, "Some other content")?;

        let mut file3 = File::create(&file3_path)?;
        writeln!(
            file3,
            r#"# Import development secrets
[[secrets]]

[secrets.dotenv]
provider = "bunker"
kind = "generic"
src = "vault:secret/wukong-cli/development#dotenv"
dst = ".env" 
"#
        )?;

        let available_files = vec![file1_path.clone(), file2_path.clone(), file3_path.clone()];
        let secret_infos = extract_secret_infos(available_files).unwrap();

        assert_eq!(secret_infos.len(), 2);

        let has_file1_path = secret_infos
            .iter()
            .any(|(path, _)| path == &file1_path.to_string_lossy().to_string());

        let does_not_have_file2_path = secret_infos
            .iter()
            .any(|(path, _)| path == &file2_path.to_string_lossy().to_string());

        let has_file3_path = secret_infos
            .iter()
            .any(|(path, _)| path == &file3_path.to_string_lossy().to_string());

        assert!(has_file1_path);
        assert!(!does_not_have_file2_path);
        assert!(has_file3_path);

        let (_, infos) = secret_infos
            .iter()
            .find(|(path, _)| path == &file1_path.to_string_lossy().to_string())
            .unwrap();

        assert_eq!(
            infos[0].key,
            "vault:secret/wukong-cli/sandboxes#dev.secrets.exs"
        );
        assert_eq!(infos[0].name, "dev.secrets.exs");
        assert_eq!(infos[0].src, "wukong-cli/sandboxes");

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
