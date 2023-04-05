use crate::loader::new_spinner_progress_bar;
use crate::utils::annotations::VaultSecretAnnotation;
use crate::{error::CliError, services::vault::Vault, utils::annotations::read_vault_annotation};
use dialoguer::Confirm;
use dialoguer::{theme::ColorfulTheme, Select};
use difference::{Changeset, Difference};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use owo_colors::OwoColorize;
use std::{collections::HashMap, io::ErrorKind};
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

pub async fn handle_config_push(path: &Path) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("🔍 Finding config with annotation");

    let available_files = get_available_files(path)?;
    let config_files = filter_vault_secret_annotations(available_files)?;

    progress_bar.finish_and_clear();

    if config_files.is_empty() {
        eprintln!("No config files found!");
        return Ok(false);
    }

    if config_files.len() != 1 {
        println!(
            "{}",
            format!("There are ({}) config files found!", config_files.len()).bright_yellow()
        );
    }

    let vault = Vault::new();
    let vault_token = vault.get_token_or_login().await?;

    let updated_configs = get_updated_configs(&vault, &vault_token, &config_files).await?;

    if updated_configs.is_empty() {
        eprintln!("No config files need to be updated!");
        return Ok(false);
    }

    let config_to_update = select_config(&updated_configs).await;
    update_secrets(&vault, &vault_token, &config_to_update).await?;

    Ok(true)
}

async fn get_updated_configs(
    vault: &Vault,
    vault_token: &str,
    config_files: &HashMap<String, VaultSecretAnnotation>,
) -> Result<HashMap<String, VaultSecretAnnotation>, CliError> {
    // Comparing local vs remote ....
    println!("{}", "Comparing local config vs remote config...".cyan());

    let mut updated_configs = HashMap::new();

    for config_file in config_files {
        let (config_path, vault_secret_annotation) = config_file;
        let remote_secrets = vault
            .get_secrets(vault_token, &vault_secret_annotation.secret_path)
            .await?
            .data;

        let config_string =
            get_local_config_as_string(&vault_secret_annotation.destination_file, config_path)?;

        // Get only one key from hashmap
        let remote_config = remote_secrets
            .get(&vault_secret_annotation.secret_name)
            .unwrap();

        let changeset = Changeset::new(remote_config, &config_string, "\n");

        if has_diff(&changeset) {
            updated_configs.insert(config_path.clone(), vault_secret_annotation.clone());
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

async fn update_secrets(
    vault: &Vault,
    vault_token: &str,
    config_to_update: &(String, VaultSecretAnnotation),
) -> Result<(), CliError> {
    let (secret_path, vault_secret_annotation) = config_to_update;

    let local_config_string =
        get_local_config_as_string(&vault_secret_annotation.destination_file, secret_path)?;

    let mut secrets = vault
        .get_secrets(vault_token, &vault_secret_annotation.secret_path)
        .await?
        .data;

    let remote_config = secrets.get(&vault_secret_annotation.secret_name).unwrap();

    print_diff(remote_config, &local_config_string);

    let agree_to_update = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Confirm this change & push?")
        .default(false)
        .interact()?;

    // Update one key from secrets:
    secrets.insert(
        vault_secret_annotation.secret_name.clone(),
        local_config_string,
    );

    let hashmap: HashMap<&str, &str> = secrets
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    if agree_to_update {
        vault
            .update_secret(vault_token, &vault_secret_annotation.secret_path, &hashmap)
            .await?;
    }

    Ok(())
}

fn has_diff(changeset: &Changeset) -> bool {
    changeset
        .diffs
        .iter()
        .any(|diff| matches!(diff, Difference::Add(_) | Difference::Rem(_)))
}

async fn select_config(
    available_config: &HashMap<String, VaultSecretAnnotation>,
) -> (String, VaultSecretAnnotation) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(
            &available_config
                .iter()
                .map(|(config_path, annotation)| {
                    format!(
                        "{} \t {}::{}/{}#{}",
                        remove_parent_directories(config_path),
                        annotation.source,
                        annotation.engine,
                        annotation.secret_path,
                        annotation.secret_name
                    )
                })
                .collect::<Vec<String>>(),
        )
        .with_prompt("Which one do you like to push the changes?")
        .default(0)
        .report(false)
        .interact_opt()
        .unwrap();

    // Clear the config file count line:
    println!("\x1B[1A\x1B[K");

    return match selection {
        Some(index) => {
            let (config_path, annotation) = available_config.iter().nth(index).unwrap();
            (config_path.clone(), annotation.clone())
        }
        None => {
            println!("No selection made, exiting...");
            std::process::exit(0);
        }
    };
}

fn get_available_files(path: &Path) -> Result<Vec<PathBuf>, CliError> {
    let path = path.try_exists().map(|value| match value {
        true => match path.to_string_lossy() == "." {
            true => current_dir(),
            false => Ok(path.to_path_buf()),
        },
        false => Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("path '{}' does not exist", path.to_string_lossy()),
        )),
    })??;

    let available_files = get_dev_config_files(&path);

    Ok(available_files)
}

fn filter_vault_secret_annotations(
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

fn print_diff(secret_string: &str, edited_secrets_str: &str) {
    let changeset = Changeset::new(secret_string, edited_secrets_str, "\n");

    println!();

    for diff in changeset.diffs.iter() {
        match diff {
            Difference::Same(part) => println!("{}", part),
            Difference::Add(part) => println!("\x1b[32m+{}\x1b[0m", part),
            Difference::Rem(part) => println!("\x1b[31m-{}\x1b[0m", part),
        }
    }
}

fn remove_parent_directories(path: &str) -> String {
    let file = Path::new(path);
    file.components()
        .filter(|component| component.as_os_str() != "..")
        .collect::<std::path::PathBuf>()
        .to_str()
        .unwrap()
        .to_string()
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
