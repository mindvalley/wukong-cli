use crate::services::vault::client::FetchSecretsData;
use crate::{error::CliError, services::vault::Vault, utils::annotations::read_vault_annotation};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use log::debug;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io::{prelude::*, ErrorKind};
use std::{
    env::current_dir,
    fs::File,
    path::{Path, PathBuf},
};

pub async fn handle_config_pull(path: &Path) -> Result<bool, CliError> {
    let path = path.try_exists().map(|value| match value {
        true => {
            if path.to_string_lossy() == "." {
                current_dir()
            } else {
                Ok(path.to_path_buf())
            }
        }
        false => Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("path '{}' does not exist", path.to_string_lossy()),
        )),
    })??;

    let vault = Vault::new();
    let vault_token = vault.get_token_or_login().await?;
    let available_files = get_dev_config_files(&path);
    let mut has_error = false;

    for file in available_files {
        let src = std::fs::read_to_string(file.clone())?;
        let annotations = read_vault_annotation(&src);

        if !annotations.is_empty() {
            let mut secrets_cache: HashMap<String, FetchSecretsData> = HashMap::new();

            eprintln!();
            eprintln!(
                "🔍 {} annotation(s) found in {}",
                annotations.len(),
                file.to_string_lossy()
            );
            for annotation in annotations {
                if annotation.key == "wukong.mindvalley.dev/config-secrets-location" {
                    if annotation.source != "vault" {
                        debug!("Invalid source: {}", annotation.source);
                        continue;
                    }
                    if annotation.engine != "secret" {
                        debug!("Invalid engine: {}", annotation.engine);
                        continue;
                    }

                    let secret_path = annotation.secret_path.clone();
                    let local_secret_path = annotation.destination_file.clone();

                    let file_path = file.parent().unwrap().join(local_secret_path.clone());

                    // cache the secrets so we don't call vault api multiple times for the same
                    // path
                    let secret = match secrets_cache.get(&secret_path) {
                        Some(secrets) => match secrets.data.get(&annotation.secret_name) {
                            Some(secret) => secret.to_string(),
                            None => {
                                debug!("Secret not found: {:?}", annotation.secret_name);
                                eprintln!(
                                    "\t{} {} {} {}",
                                    "Not created".red(),
                                    file_path.to_string_lossy(),
                                    "because".bold(),
                                    "Secret not found".bold().red()
                                );
                                has_error = true;
                                continue;
                            }
                        },
                        None => {
                            let secrets = match vault.get_secrets(&vault_token, &secret_path).await
                            {
                                Ok(secrets) => secrets,
                                Err(err) => {
                                    debug!("Error while fetching secrets: {:?}", &secret_path);
                                    eprintln!(
                                        "\t{} {} {} {}",
                                        "Not created".red(),
                                        file_path.to_string_lossy(),
                                        "because".bold(),
                                        err.bold().red()
                                    );
                                    has_error = true;
                                    continue;
                                }
                            };
                            secrets_cache.insert(secret_path.clone(), secrets);

                            match secrets_cache
                                .get(&secret_path)
                                .unwrap()
                                .data
                                .get(&annotation.secret_name)
                            {
                                Some(secret) => secret.to_string(),
                                None => {
                                    debug!("Secret not found: {:?}", annotation.secret_name);
                                    eprintln!(
                                        "\t{} {} {} {}",
                                        "Not created".red(),
                                        file_path.to_string_lossy(),
                                        "because".bold(),
                                        "Secret not found".bold().red()
                                    );
                                    has_error = true;
                                    continue;
                                }
                            }
                        }
                    };

                    if local_secret_path.contains('/') {
                        let dir_path = file_path.parent().unwrap();
                        if let Err(err) = std::fs::create_dir_all(dir_path) {
                            debug!("Error while creating directory: {:?}", err);
                            eprintln!(
                                "\t{} {} {} {}",
                                "Not created".red(),
                                file_path.to_string_lossy(),
                                "because".bold(),
                                err.to_string().bold().red()
                            );
                            has_error = true;
                            continue;
                        };
                    }

                    match File::create(&file_path) {
                        Ok(mut file) => {
                            if let Err(err) = file.write_all(secret.as_bytes()) {
                                debug!("Error while creating file: {:?}", err);
                                eprintln!(
                                    "\t{} {} {} {}",
                                    "Not created".red(),
                                    file_path.to_string_lossy(),
                                    "because".bold(),
                                    err.to_string().bold().red()
                                );
                                has_error = true;
                                continue;
                            };
                        }
                        Err(err) => {
                            debug!("Error while writing file: {:?}", err);
                            eprintln!(
                                "\t{} {} {} {}",
                                "Not created".red(),
                                file_path.to_string_lossy(),
                                "because".bold(),
                                err.to_string().bold().red()
                            );
                            has_error = true;
                            continue;
                        }
                    }

                    eprintln!("\t{} {}", "Created".green(), file_path.to_string_lossy());
                }
            }
        } else {
            eprintln!("🔍 No annotation found in {}", file.to_string_lossy());
        }
    }

    if has_error {
        Ok(false)
    } else {
        Ok(true)
    }
}

fn get_dev_config_files(path: &Path) -> Vec<PathBuf> {
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

#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::prelude::{FileTouch, PathChild};

    #[test]
    fn test_dev_files() {
        // three files:
        // temp/config/dev.exs
        // temp/config/prod.exs
        // temp/app/config/dev.exs

        let temp = assert_fs::TempDir::new().unwrap();
        let dev_config_file = temp.child("config/dev.exs");
        dev_config_file.touch().unwrap();
        let prod_config_file = temp.child("config/prod.exs");
        prod_config_file.touch().unwrap();
        let another_dev_config_file = temp.child("app/config/dev.exs");
        another_dev_config_file.touch().unwrap();

        let files = get_dev_config_files(&temp.to_path_buf());
        let files_names = files
            .iter()
            .map(|f| f.to_string_lossy())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 2);
        assert!(files_names.contains(&dev_config_file.path().to_string_lossy()));
        assert!(files_names.contains(&another_dev_config_file.path().to_string_lossy()));

        temp.close().unwrap();
    }
}
