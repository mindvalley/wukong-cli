use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use owo_colors::OwoColorize;
use wukong_sdk::secret_extractors::SecretInfo;

use crate::{
    commands::{dev::config::utils::make_path_relative, Context},
    config::Config,
    error::{DevConfigError, WKCliError},
    loader::new_spinner,
    output::colored_println,
    wukong_client::WKClient,
};

use super::{
    diff::print_diff,
    utils::{
        extract_secret_infos, get_local_config_path, get_secret_config_files, get_updated_configs,
        parse_wukong_src, vault_token_for,
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
        return Err(WKCliError::DevConfigError(DevConfigError::ConfigNotFound));
    }

    if extracted_infos.len() != 1 {
        println!(
            "{}",
            format!("There are ({}) config files found!", extracted_infos.len()).bright_yellow()
        );
    }

    let mut config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;
    let vault_token = vault_token_for(&extracted_infos, &mut config).await?;

    let updated_configs =
        get_updated_configs(&mut wk_client, &vault_token, &extracted_infos).await?;

    if updated_configs.is_empty() {
        println!(
            "The config file is already up to date with the remote. There are no changes to push."
        );

        return Ok(true);
    }

    if updated_configs.len() == 1 {
        println!(
            "{}",
            "There is only one config file to update...".bright_yellow()
        );
        let (annotation, remote_config, local_config, config_path) =
            updated_configs.first().unwrap();

        update_secrets(
            &mut wk_client,
            &vault_token,
            annotation,
            config_path,
            remote_config,
            local_config,
        )
        .await?;
    } else {
        let (annotation, remote_config, local_config, config_path) =
            select_config(&updated_configs).await;
        update_secrets(
            &mut wk_client,
            &vault_token,
            annotation,
            &config_path,
            &remote_config,
            &local_config,
        )
        .await?;
    }

    Ok(true)
}

async fn update_secrets(
    wk_client: &mut WKClient,
    vault_token: &str,
    secret_info: &SecretInfo,
    config_path: &str,
    remote_config: &str,
    local_config_string: &str,
) -> Result<(), WKCliError> {
    let local_config_path = get_local_config_path(config_path, &secret_info.destination_file);

    print_diff(
        remote_config,
        local_config_string,
        &local_config_path.to_string_lossy(),
    );

    let agree_to_update = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Confirm this change & push?")
        .default(false)
        .interact()?;

    let mut secrets_ref: HashMap<&str, &str> = HashMap::new();
    secrets_ref.insert(&secret_info.name, local_config_string);

    if agree_to_update {
        let loader = new_spinner();
        loader.set_message("Updating secrets... ");

        if secret_info.provider == "wukong" {
            let (app, ns, path) = parse_wukong_src(&secret_info.src);
            wk_client
                .update_wukong_secrets(&app, &ns, &path, &secrets_ref)
                .await?;
        } else {
            wk_client
                .update_secret(vault_token, &secret_info.src, &secrets_ref)
                .await?;
        }

        colored_println!("Successfully updated the secrets.");
    }

    Ok(())
}

async fn select_config<'a>(
    available_config: &[(&'a SecretInfo, String, String, String)],
) -> (&'a SecretInfo, String, String, String) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(
            &available_config
                .iter()
                .map(|(secret_info, _, _, config_path)| {
                    let local_config_path =
                        get_local_config_path(config_path, &secret_info.destination_file);

                    let remote_display = if secret_info.provider == "wukong" {
                        format!("wukong:{}#{}", secret_info.src, secret_info.name)
                    } else {
                        format!("vault:secret/{}#{}", secret_info.src, secret_info.name)
                    };

                    format!(
                        "{:<50}{}",
                        make_path_relative(&local_config_path.to_string_lossy()),
                        remote_display,
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
            let (secret_info, remote_config, local_config, config_path) =
                available_config.get(index).unwrap();
            (
                secret_info,
                remote_config.clone(),
                local_config.clone(),
                config_path.clone(),
            )
        }
        None => {
            println!("No selection made, exiting...");
            std::process::exit(0);
        }
    };
}
