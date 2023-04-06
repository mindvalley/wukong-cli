use crate::error::DevConfigError;
use crate::loader::new_spinner_progress_bar;
use crate::utils::annotations::VaultSecretAnnotation;
use crate::{error::CliError, services::vault::Vault};
use dialoguer::Confirm;
use dialoguer::{theme::ColorfulTheme, Select};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::path::PathBuf;

use super::diff::print_diff;
use super::utils::{
    filter_config_with_secret_annotations, get_available_files, get_updated_configs,
    remove_parent_directories,
};

pub async fn handle_config_push() -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("🔍 Finding config with annotation");

    let available_files = get_available_files()?;
    let config_files = filter_config_with_secret_annotations(available_files)?;

    progress_bar.finish_and_clear();

    if config_files.is_empty() {
        return Err(CliError::DevConfigError(DevConfigError::ConfigNotFound));
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
        println!(
            "The config file is already up to date with the Vault Bunker. There are no changes to push."
        );

        return Ok(true);
    }

    if updated_configs.len() == 1 {
        println!(
            "{}",
            "There is only one config file to update...".bright_yellow()
        );
        let (config_path, (annotation, _, _)) = updated_configs.iter().next().unwrap();

        update_secrets(
            &vault,
            &vault_token,
            &(config_path.clone(), annotation.clone()),
        )
        .await?;
    } else {
        let config_to_update = select_config(&updated_configs).await;
        update_secrets(&vault, &vault_token, &config_to_update).await?;
    }

    Ok(true)
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

    print_diff(remote_config, &local_config_string, secret_path);

    let agree_to_update = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Confirm this change & push?")
        .default(false)
        .interact()?;

    // Update one key from secrets:
    secrets.insert(
        vault_secret_annotation.secret_name.clone(),
        local_config_string,
    );

    let secrets_ref: HashMap<&str, &str> = secrets
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    if agree_to_update {
        vault
            .update_secret(
                vault_token,
                &vault_secret_annotation.secret_path,
                &secrets_ref,
            )
            .await?;
    }

    Ok(())
}

async fn select_config(
    available_config: &HashMap<String, (VaultSecretAnnotation, String, String)>,
) -> (String, VaultSecretAnnotation) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(
            &available_config
                .iter()
                .map(|(config_path, (annotation, _, _))| {
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
            let (config_path, (annotation, _, _)) = available_config.iter().nth(index).unwrap();
            (config_path.clone(), annotation.clone())
        }
        None => {
            println!("No selection made, exiting...");
            std::process::exit(0);
        }
    };
}
