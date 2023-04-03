use crate::loader::new_spinner_progress_bar;
use crate::{error::CliError, services::vault::Vault, utils::annotations::read_vault_annotation};
use dialoguer::Confirm;
use dialoguer::{theme::ColorfulTheme, Select};
use difference::{Changeset, Difference};
use edit::Builder;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use owo_colors::OwoColorize;
use std::{collections::HashMap, io::ErrorKind};
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

pub async fn handle_config_secrets_edit(path: &Path) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("🔍 Finding config with annotation");

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
    let mut annotations_selections: Vec<(String, String)> = vec![];

    for file in available_files {
        let src = std::fs::read_to_string(file.clone())?;
        let annotations = read_vault_annotation(&src);

        // Push this to annotations selections:
        annotations_selections.extend(
            annotations
                .iter()
                .filter(|annotation| {
                    annotation.key == "wukong.mindvalley.dev/config-secrets-location"
                        && annotation.source == "vault"
                        && annotation.engine == "secret"
                })
                .map(|annotation| {
                    (
                        annotation.secret_path.clone(),
                        format!(
                            "{} \t {}::{}/{}#{}",
                            remove_parent_directories(&file),
                            &annotation.source,
                            &annotation.engine,
                            &annotation.secret_path,
                            &annotation.secret_name
                        ),
                    )
                })
                .collect::<Vec<(String, String)>>(),
        )
    }

    progress_bar.finish_and_clear();

    if annotations_selections.is_empty() {
        eprintln!("No config files found !");
        return Ok(false);
    }

    println!(
        "{}",
        format!(
            "There are ({}) config files found!",
            annotations_selections.len()
        )
        .bright_yellow()
    );

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(
            &annotations_selections
                .iter()
                .map(|(_, value)| value)
                .collect::<Vec<&String>>(),
        )
        .with_prompt("Which one do you like to make the changes ?")
        .default(0)
        .report(false)
        .interact_opt()
        .unwrap();

    // Clear the config file count line:
    println!("\x1B[1A\x1B[K");

    match selection {
        Some(index) => {
            let vault = Vault::new();
            let vault_token = vault.get_token_or_login().await?;

            // Get the secret path based on the selection:
            let (secret_path, _) = &annotations_selections[index];
            let secrets = vault.get_secrets(&vault_token, &secret_path).await?.data;

            let secret_string = serde_json::to_string_pretty::<HashMap<String, String>>(&secrets)?;

            // Open editor with secrets:
            let edited_secrets_str = edit::edit_with_builder(
                serde_json::to_string_pretty::<HashMap<String, String>>(&secrets)?,
                Builder::new().prefix("config_secrets_edit").suffix(".json"),
            )?;

            // Intentionally placed here to throw json parse error if the user input is invalid:
            let edited_secrets: HashMap<&str, &str> = serde_json::from_str(&edited_secrets_str)?;

            println!(
                "{}",
                "Finished editing, please review your changes before pusing to Bunker...".cyan()
            );
            print_diff(&secret_string, &edited_secrets_str);

            let agree_to_deploy = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure to push this change?")
                .default(false)
                .interact()?;

            if agree_to_deploy {
                vault
                    .update_secret(&vault_token, secret_path, &edited_secrets)
                    .await?;
            }
        }
        None => {
            return Ok(false);
        }
    }

    Ok(true)
}

fn print_diff(secret_string: &str, edited_secrets_str: &str) -> () {
    let changeset = Changeset::new(&edited_secrets_str, &secret_string, "\n");

    for diff in &changeset.diffs {
        match diff {
            Difference::Same(part) => println!("{}", part),
            Difference::Add(part) => println!("\x1b[32m+{}\x1b[0m", part),
            Difference::Rem(part) => println!("\x1b[31m-{}\x1b[0m", part),
        }
    }
}

fn remove_parent_directories(file: &Path) -> String {
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
