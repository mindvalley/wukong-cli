use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use owo_colors::OwoColorize;
use wukong_sdk::secret_extractors::SecretInfo;

use crate::{
    auth::vault,
    commands::{dev::config::utils::make_path_relative, Context},
    config::Config,
    error::WKCliError,
    loader::new_spinner,
    wukong_client::WKClient,
};

use super::{
    diff::print_diff,
    utils::{
        extract_secret_infos, get_local_config_as_string, get_local_config_path,
        get_secret_config_files, get_updated_configs,
    },
};
use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[wukong_telemetry(command_event = "dev_config_push")]
pub async fn handle_config_push(context: Context) -> Result<bool, WKCliError> {
    let loader = new_spinner();
    loader.set_message("🔍 Finding config with annotation");

    let secret_config_files = get_secret_config_files(None)?;
    let extracted_infos = extract_secret_infos(secret_config_files)?;

    loader.finish_and_clear();

    if extracted_infos.is_empty() {
        return Err(WKCliError::DevConfigNotFound);
    }

    if extracted_infos.len() != 1 {
        println!(
            "{}",
            format!("There are ({}) config files found!", extracted_infos.len()).bright_yellow()
        );
    }

    let mut config = Config::load_from_default_path()?;
    let wk_client = WKClient::new(&config);
    let vault_token = vault::get_token_or_login(&mut config).await?;

    let updated_configs = get_updated_configs(&wk_client, &vault_token, &extracted_infos).await?;

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

        update_secrets(&wk_client, &vault_token, &(config_path.clone(), annotation)).await?;
    } else {
        let config_to_update = select_config(&updated_configs).await;
        update_secrets(&wk_client, &vault_token, &config_to_update).await?;
    }

    Ok(true)
}

async fn update_secrets(
    wk_client: &WKClient,
    vault_token: &str,
    config_to_update: &(String, &SecretInfo),
) -> Result<(), WKCliError> {
    let (config_path, secret_info) = config_to_update;

    let local_config_string =
        get_local_config_as_string(&secret_info.destination_file, config_path)?;

    let remote_config = wk_client
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
        wk_client
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
