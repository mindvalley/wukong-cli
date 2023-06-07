use crate::commands::dev::config::utils::get_local_config_path;
use crate::services::vault::client::FetchSecretsData;
use crate::{error::CliError, services::vault::Vault};
use log::debug;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io::{prelude::*, ErrorKind};
use std::{env::current_dir, fs::File, path::Path};

use super::utils::{extract_secret_infos, get_secret_config_files};

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

    let secret_config_files = get_secret_config_files(Some(path))?;
    let extracted_infos = extract_secret_infos(secret_config_files);

    let vault = Vault::new();
    let vault_token = vault.get_token_or_login().await?;
    let mut has_error = false;

    let mut secrets_cache: HashMap<String, FetchSecretsData> = HashMap::new();
    for info in extracted_infos {
        eprintln!();
        eprintln!("🔍 {} annotation(s) found in {}", info.1.len(), info.0);

        for annotation in info.1 {
            let source_path = annotation.src.clone();
            let destination_path = annotation.destination_file.clone();

            let file_path = get_local_config_path(&destination_path, &info.0);

            // cache the secrets so we don't call vault api multiple times for the same
            // path
            let secret = match secrets_cache.get(&source_path) {
                Some(secrets) => match secrets.data.get(&annotation.name) {
                    Some(secret) => secret.to_string(),
                    None => {
                        debug!("Secret not found: {:?}", annotation.name);
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
                    let secrets = match vault.get_secrets(&vault_token, &source_path).await {
                        Ok(secrets) => secrets,
                        Err(err) => {
                            debug!("Error while fetching secrets: {:?}", &source_path);
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
                    secrets_cache.insert(source_path.clone(), secrets);

                    match secrets_cache
                        .get(&source_path)
                        .unwrap()
                        .data
                        .get(&annotation.name)
                    {
                        Some(secret) => secret.to_string(),
                        None => {
                            debug!("Secret not found: {:?}", annotation.name);
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

            if destination_path.contains('/') {
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

    if has_error {
        Ok(false)
    } else {
        Ok(true)
    }
}

// fn get_secret_config_files(path: &Path) -> Vec<PathBuf> {
//     let mut overrides = OverrideBuilder::new(path);
//     overrides.add("**/config/dev.exs").unwrap();
//     overrides.add("**/.wukong.toml").unwrap();
//
//     WalkBuilder::new(path)
//         .overrides(overrides.build().unwrap())
//         .build()
//         .flatten()
//         .filter(|e| e.path().is_file())
//         .map(|e| e.path().to_path_buf())
//         .collect()
// }

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

        let files = get_secret_config_files(&temp.to_path_buf()).unwrap();
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
