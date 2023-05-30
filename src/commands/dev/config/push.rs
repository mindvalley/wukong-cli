use crate::error::DevConfigError;
use crate::loader::new_spinner_progress_bar;
use crate::utils::annotations::VaultSecretAnnotation;
use crate::{error::CliError, services::vault::Vault};
use dialoguer::Confirm;
use dialoguer::{theme::ColorfulTheme, Select};
use owo_colors::OwoColorize;
use std::collections::HashMap;

use super::diff::print_diff;
use super::utils::{
    filter_config_with_secret_annotations, get_dev_config_files, get_local_config_as_string,
    get_local_config_path, get_updated_configs, make_path_relative,
};

pub async fn handle_config_push() -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("🔍 Finding config with annotation");

    let dev_config_files = get_dev_config_files()?;
    let dev_config_with_secret_annotations =
        filter_config_with_secret_annotations(dev_config_files)?;

    progress_bar.finish_and_clear();

    if dev_config_with_secret_annotations.is_empty() {
        return Err(CliError::DevConfigError(DevConfigError::ConfigNotFound));
    }

    if dev_config_with_secret_annotations.len() != 1 {
        println!(
            "{}",
            format!(
                "There are ({}) config files found!",
                dev_config_with_secret_annotations.len()
            )
            .bright_yellow()
        );
    }

    let vault = Vault::new();
    let vault_token = vault.get_token_or_login().await?;

    let updated_configs =
        get_updated_configs(&vault, &vault_token, &dev_config_with_secret_annotations).await?;

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
        let (annotation, _, _, config_path) = updated_configs.first().unwrap();

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

async fn update_secrets(
    vault: &Vault,
    vault_token: &str,
    config_to_update: &(String, VaultSecretAnnotation),
) -> Result<(), CliError> {
    let (config_path, vault_secret_annotation) = config_to_update;

    let local_config_string =
        get_local_config_as_string(&vault_secret_annotation.destination_file, config_path)?;

    let remote_config = vault
        .get_secret(
            vault_token,
            &vault_secret_annotation.secret_path,
            &vault_secret_annotation.secret_name,
        )
        .await?;

    let local_config_path =
        get_local_config_path(config_path, &vault_secret_annotation.destination_file);

    print_diff(
        &remote_config,
        &local_config_string,
        &local_config_path.to_string_lossy(),
    );

    let agree_to_update = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Confirm this change & push?")
        .default(false)
        .interact()?;

    let mut secrets_ref: HashMap<&str, &str> = HashMap::new();
    secrets_ref.insert(&vault_secret_annotation.secret_name, &local_config_string);

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
    available_config: &[(VaultSecretAnnotation, String, String, String)],
) -> (String, VaultSecretAnnotation) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(
            &available_config
                .iter()
                .map(|(annotation, _, _, config_path)| {
                    let local_config_path =
                        get_local_config_path(config_path, &annotation.destination_file);

                    format!(
                        "{:<50}{}::{}/{}#{}",
                        make_path_relative(&local_config_path.to_string_lossy()),
                        annotation.source,
                        annotation.engine,
                        annotation.secret_path,
                        annotation.secret_name,
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
            let (annotation, _, _, config_path) = available_config.get(index).unwrap();
            (config_path.clone(), annotation.clone())
        }
        None => {
            println!("No selection made, exiting...");
            std::process::exit(0);
        }
    };
}
