use std::{
    collections::HashMap,
    env::current_dir,
    fs::File,
    io::{ErrorKind, Write},
    path::Path,
};

use log::debug;
use owo_colors::OwoColorize;
use wukong_sdk::services::vault::client::FetchSecretsData;

use crate::{
    auth::vault,
    commands::{dev::config::utils::get_local_config_path, Context},
    config::Config,
    error::WKCliError,
    wukong_client::WKClient,
};

use super::utils::{extract_secret_infos, get_secret_config_files};
use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[wukong_telemetry(command_event = "dev_config_pull")]
pub async fn handle_config_pull(context: Context, path: &Path) -> Result<bool, WKCliError> {
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
    let extracted_infos = extract_secret_infos(secret_config_files)?;

    let mut config = Config::load_from_default_path()?;
    let wk_client = WKClient::new(&config)?;
    let vault_token = vault::get_token_or_login(&mut config).await?;

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
                    let secrets = match wk_client.get_secrets(&vault_token, &source_path).await {
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
