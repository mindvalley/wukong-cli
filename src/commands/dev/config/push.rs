use crate::error::DevConfigError;
use crate::loader::new_spinner_progress_bar;
use crate::utils::annotations::VaultSecretAnnotation;
use crate::{
    error::CliError, services::vault::Vault, utils::annotations::read_vault_annotation,
    utils::line::Line,
};
use dialoguer::console::{style, Style};
use dialoguer::Confirm;
use dialoguer::{theme::ColorfulTheme, Select};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use log::debug;
use owo_colors::OwoColorize;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
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
    }

    if updated_configs.len() == 1 {
        println!(
            "{}",
            "There is only one config file to update...".bright_yellow()
        );
        let (config_path, annotation) = updated_configs.iter().next().unwrap();

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

async fn get_updated_configs(
    vault: &Vault,
    vault_token: &str,
    config_files: &HashMap<String, VaultSecretAnnotation>,
) -> Result<HashMap<String, VaultSecretAnnotation>, CliError> {
    // Comparing local vs remote ....
    println!("{}", "comparing local config vs remote config...".cyan());

    let mut updated_configs = HashMap::new();

    for config_file in config_files {
        let (config_path, vault_secret_annotation) = config_file;
        let remote_secrets = vault
            .get_secrets(vault_token, &vault_secret_annotation.secret_path)
            .await?
            .data;

        // Handle and throw InvalidSecretPath if No such file or directory (os error 2):
        let config_string =
            get_local_config_as_string(&vault_secret_annotation.destination_file, config_path)
                .map_err(|error| {
                    debug!("Error: {:?}", error);
                    CliError::DevConfigError(DevConfigError::ConfigSecretNotFound)
                })?;

        // Get only one key from hashmap
        let remote_config = match remote_secrets.get(&vault_secret_annotation.secret_name) {
            Some(config) => config,
            None => {
                return Err(CliError::DevConfigError(
                    DevConfigError::InvalidSecretPath {
                        path: vault_secret_annotation.secret_name.clone(),
                    },
                ))
            }
        };

        if has_diff(remote_config, &config_string) {
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

fn has_diff(old_secret_config: &str, new_secret_config: &str) -> bool {
    let changeset = TextDiff::from_lines(old_secret_config, new_secret_config);

    changeset
        .iter_all_changes()
        .any(|change| matches!(change.tag(), ChangeTag::Delete | ChangeTag::Insert))
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

fn get_available_files() -> Result<Vec<PathBuf>, CliError> {
    let current_path = current_dir()?;
    let available_files = get_dev_config_files(&current_path);

    Ok(available_files)
}

fn filter_config_with_secret_annotations(
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

fn print_diff(old_secret_config: &str, new_secret_config: &str, secret_path: &str) {
    println!();
    println!("{}", secret_path.dimmed());

    let diff = TextDiff::from_lines(old_secret_config, new_secret_config);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("{:-^1$}", "-", 80);
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                print!(
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                );
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        print!("{}", s.apply_to(value).underlined().on_black());
                    } else {
                        print!("{}", s.apply_to(value));
                    }
                }
                if change.missing_newline() {
                    println!();
                }
            }
        }
    }

    println!();
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

// Test:
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_string() {
        let path = "";
        let result = remove_parent_directories(path);
        assert_eq!(result, "");
    }

    #[test]
    fn test_no_parent_directories() {
        let path = "dir1/dir2/file.txt";
        let result = remove_parent_directories(path);
        assert_eq!(result, "dir1/dir2/file.txt");
    }

    #[test]
    fn test_single_parent_directory() {
        let path = "dir1/../dir2/file.txt";
        let result = remove_parent_directories(path);
        assert_eq!(result, "dir1/dir2/file.txt");
    }

    #[test]
    fn test_invalid_characters() {
        let path = "dir1/inv@lid/../dir2/file.txt";
        let result = remove_parent_directories(path);
        assert_eq!(result, "dir1/inv@lid/dir2/file.txt");
    }

    #[test]
    fn test_non_existent_file() {
        let non_existent_path = "non_existent_file.txt";

        let result = get_local_config_as_string("destination_file", non_existent_path);
        assert!(result.is_err());
    }
}
