use crate::error::DevConfigError;
use crate::loader::new_spinner_progress_bar;
use crate::utils::secret_extractors::SecretInfo;
use crate::{error::WKError, services::vault::Vault};
use dialoguer::Confirm;
use dialoguer::{theme::ColorfulTheme, Select};
use owo_colors::OwoColorize;
use std::collections::HashMap;

use super::diff::print_diff;
use super::utils::{
    extract_secret_infos, get_local_config_as_string, get_local_config_path,
    get_secret_config_files, get_updated_configs, make_path_relative,
};

pub async fn handle_config_push() -> Result<bool, WKError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("🔍 Finding config with annotation");

    let secret_config_files = get_secret_config_files(None)?;
    let extracted_infos = extract_secret_infos(secret_config_files)?;

    progress_bar.finish_and_clear();

    if extracted_infos.is_empty() {
        return Err(WKError::DevConfigError(DevConfigError::ConfigNotFound));
    }

    if extracted_infos.len() != 1 {
        println!(
            "{}",
            format!("There are ({}) config files found!", extracted_infos.len()).bright_yellow()
        );
    }

    let vault = Vault::new();
    let vault_token = vault.get_token_or_login().await?;

    let updated_configs = get_updated_configs(&vault, &vault_token, &extracted_infos).await?;

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

        update_secrets(&vault, &vault_token, &(config_path.clone(), annotation)).await?;
    } else {
        let config_to_update = select_config(&updated_configs).await;
        update_secrets(&vault, &vault_token, &config_to_update).await?;
    }

    Ok(true)
}

async fn update_secrets(
    vault: &Vault,
    vault_token: &str,
    config_to_update: &(String, &SecretInfo),
) -> Result<(), WKError> {
    let (config_path, secret_info) = config_to_update;

    let local_config_string =
        get_local_config_as_string(&secret_info.destination_file, config_path)?;

    let remote_config = vault
        .get_secret(vault_token, &secret_info.src, &secret_info.name)
        .await?;

    let local_config_path = get_local_config_path(config_path, &secret_info.destination_file);

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
    secrets_ref.insert(&secret_info.name, &local_config_string);

    if agree_to_update {
        vault
            .update_secret(vault_token, &secret_info.src, &secrets_ref)
            .await?;
    }

    Ok(())
}

async fn select_config<'a>(
    available_config: &[(&'a SecretInfo, String, String, String)],
) -> (String, &'a SecretInfo) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(
            &available_config
                .iter()
                .map(|(secret_info, _, _, config_path)| {
                    let local_config_path =
                        get_local_config_path(config_path, &secret_info.destination_file);

                    format!(
                        "{:<50}vault:secret/{}#{}",
                        make_path_relative(&local_config_path.to_string_lossy()),
                        secret_info.src,
                        secret_info.name,
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
            let (secret_info, _, _, config_path) = available_config.get(index).unwrap();
            (config_path.clone(), secret_info)
        }
        None => {
            println!("No selection made, exiting...");
            std::process::exit(0);
        }
    };
}
