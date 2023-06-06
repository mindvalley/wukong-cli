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
use toml::Value;

struct SecretInfo {
    provider: String,
    kind: String,
    src: String,
    destination_file: String,
    name: String,
    annotated_file: PathBuf,
}

trait Extractor {
    fn extract(&self, file: &PathBuf) -> Vec<SecretInfo>;
}

struct WukongTomlExtractor;
impl Extractor for WukongTomlExtractor {
    fn extract(&self, file: &PathBuf) -> Vec<SecretInfo> {
        let toml_string = std::fs::read_to_string(file).expect("Failed to read config file");

        // Parse the TOML string
        let parsed_toml: Value = toml::from_str(&toml_string).expect("Failed to parse TOML");

        // Access values from the parsed TOML
        let secrets = parsed_toml.get("secrets").expect("secrets not found");

        let mut extracted = vec![];

        if let Some(secrets_array) = secrets.as_array() {
            for secret in secrets_array {
                if let Some(secret_table) = secret.as_table() {
                    for key in secret_table.keys() {
                        let secret_data = secret_table.get(key).unwrap();

                        let source = secret_data["src"].as_str().unwrap().to_string();
                        let value_part = source.split('#').collect::<Vec<&str>>();
                        if value_part.len() != 2 {
                            continue;
                        }
                        let source = value_part[0].to_string();
                        let secret_name = value_part[1].to_string();

                        let splited_source_and_path = source.split(':').collect::<Vec<&str>>();
                        if splited_source_and_path.len() != 2 {
                            continue;
                        }
                        let path_with_engine = splited_source_and_path[1].to_string();

                        let splited_engine_and_path =
                            path_with_engine.split('/').collect::<Vec<&str>>();
                        let (_engine, path) = splited_engine_and_path.split_at(1);

                        let src = path.join("/");

                        extracted.push(SecretInfo {
                            provider: secret_data["provider"].as_str().unwrap().to_string(),
                            kind: secret_data["kind"].as_str().unwrap().to_string(),
                            src,
                            destination_file: secret_data["dst"].as_str().unwrap().to_string(),
                            name: secret_name,
                            annotated_file: file.clone(),
                        });
                    }
                }
            }
        }

        extracted
    }
}

struct ElixirAnnotationExtractor;
impl Extractor for ElixirAnnotationExtractor {
    fn extract(&self, file: &PathBuf) -> Vec<SecretInfo> {
        let src = std::fs::read_to_string(file.clone()).unwrap();
        let annotations = read_vault_annotation(&src);

        let mut extracted = vec![];

        if !annotations.is_empty() {
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

                    extracted.push(SecretInfo {
                        provider: "bunker".to_string(),
                        kind: "elixir_config".to_string(),
                        src: annotation.secret_path.clone(),
                        destination_file: annotation.destination_file.clone(),
                        name: annotation.secret_name.clone(),
                        annotated_file: file.clone(),
                    });
                }
            }
        } else {
            eprintln!("🔍 No annotation found in {}", file.to_string_lossy());
        }

        extracted
    }
}

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

    let wukong_toml_files = get_wukong_toml_file(&path);

    let wukong_toml_file_extractor = WukongTomlExtractor {};
    let wk_toml_extracted: Vec<(PathBuf, Vec<SecretInfo>)> = wukong_toml_files
        .iter()
        .map(|file| (file.clone(), wukong_toml_file_extractor.extract(&file)))
        .collect();

    let available_files = get_dev_config_files(&path);

    let vault = Vault::new();
    let vault_token = vault.get_token_or_login().await?;
    let mut has_error = false;

    let elixir_annotation_extractor = ElixirAnnotationExtractor {};
    let elixir_extracted: Vec<(PathBuf, Vec<SecretInfo>)> = available_files
        .iter()
        .map(|file| (file.clone(), elixir_annotation_extractor.extract(&file)))
        .collect();

    let extracted_infos = wk_toml_extracted
        .into_iter()
        .chain(elixir_extracted)
        .collect::<Vec<(PathBuf, Vec<SecretInfo>)>>();

    let mut secrets_cache: HashMap<String, FetchSecretsData> = HashMap::new();
    for info in extracted_infos {
        eprintln!();
        eprintln!(
            "🔍 {} annotation(s) found in {}",
            info.1.len(),
            info.0.to_string_lossy()
        );

        for annotation in info.1 {
            let source_path = annotation.src.clone();
            let destination_path = annotation.destination_file.clone();

            let file_path = info.0.parent().unwrap().join(destination_path.clone());

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

fn get_wukong_toml_file(path: &Path) -> Vec<PathBuf> {
    let mut overrides = OverrideBuilder::new(path);
    overrides.add("**/.wukong.toml").unwrap();

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
