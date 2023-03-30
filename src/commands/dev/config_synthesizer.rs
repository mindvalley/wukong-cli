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

pub async fn handle_config_synthesizer(path: &Path) -> Result<bool, CliError> {
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

    for file in available_files {
        let src = std::fs::read_to_string(file.clone())?;
        let annotations = read_vault_annotation(&src);

        if !annotations.is_empty() {
            let mut secrets_cache: HashMap<String, FetchSecretsData> = HashMap::new();

            eprintln!(
                "🔍 {} annotation(s) found in {}",
                annotations.len(),
                file.to_string_lossy()
            );
            for annotation in annotations {
                if annotation.key == "wukong.mindvalley.dev/config-secrets-location" {
                    let secret_path = annotation.secret_path.clone();
                    let local_secret_path = annotation.destination_file.clone();

                    let vault_path_part = secret_path.split(':').collect::<Vec<&str>>();
                    let vault_secret_path = vault_path_part[1].to_string();

                    let file_path = file.parent().unwrap().join(local_secret_path.clone());

                    // cache the secrets so we don't call vault api multiple times for the same
                    // path
                    let secret = match secrets_cache.get(&vault_secret_path) {
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
                                continue;
                            }
                        },
                        None => {
                            let secrets =
                                match vault.get_secrets(&vault_token, &vault_secret_path).await {
                                    Ok(secrets) => secrets,
                                    Err(err) => {
                                        debug!(
                                            "Error while fetching secrets: {:?}",
                                            &vault_secret_path
                                        );
                                        eprintln!(
                                            "\t{} {} {} {}",
                                            "Not created".red(),
                                            file_path.to_string_lossy(),
                                            "because".bold(),
                                            err.bold().red()
                                        );
                                        continue;
                                    }
                                };
                            secrets_cache.insert(vault_secret_path.clone(), secrets);

                            match secrets_cache
                                .get(&vault_secret_path)
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
                            continue;
                        }
                    }

                    eprintln!("\t{} {}", "Created".green(), file_path.to_string_lossy());
                }
            }
        }
    }

    Ok(true)
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
        assert_eq!(files.len(), 2);
        assert_eq!(
            files[0].to_string_lossy(),
            another_dev_config_file.path().to_string_lossy()
        );
        assert_eq!(
            files[1].to_string_lossy(),
            dev_config_file.path().to_string_lossy()
        );

        temp.close().unwrap();
    }
}
