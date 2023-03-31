use crate::{
    error::CliError, services::vault::Vault, utils::custom_dialoguer_theme::CustomDialoguerTheme,
};
use dialoguer::{
    console::{style, Style},
    MultiSelect,
};
use difference::{Changeset, Difference};
use edit::Builder;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use serde_json;
use std::{collections::HashMap, io::ErrorKind};
use std::{
    env::current_dir,
    fs::File,
    path::{Path, PathBuf},
};

pub async fn handle_config_secrets_edit(path: &Path) -> Result<bool, CliError> {
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

    let vault = Vault::new();
    let vault_token = vault.get_token_or_login().await?;
    let available_files = get_dev_config_files(&path);
    let secrets = vault.get_secrets(&vault_token, "wukong-test").await?;

    println!("{:?}", secrets);
    // Open editor with secrets:
    // let edited_secrets_str = edit::edit_with_builder(
    //     serde_json::to_string_pretty::<HashMap<String, String>>(&secrets)?,
    //     Builder::new().prefix("config_secrets_edit").suffix(".json"),
    // )?;

    // let edited_secrets: HashMap<String, String> = serde_json::from_str(&edited_secrets_str)?;
    // let secrets_checklist_items = generate_checklist_items(&secrets, &edited_secrets);

    // // Build the checklist:
    // let themes = CustomDialoguerTheme::default();
    // let selected_secrets = MultiSelect::with_theme(&themes)
    //     .with_prompt(format!(
    //         "{}",
    //         style("Choose which changes to update").bold()
    //     ))
    //     .items_checked(
    //         &secrets_checklist_items
    //             .iter()
    //             .map(|(value, _)| (value.to_string(), true))
    //             .collect::<Vec<(String, bool)>>(),
    //     )
    //     .report(false)
    //     .interact()?;

    // for selected_secrets in selected_secrets {
    //     let key = &secrets_checklist_items[selected_secrets].1;

    //     // Update the secret with updated value:
    //     if let Some(value) = edited_secrets.get(&key.to_string()) {
    //         secrets.insert(key.to_string(), value.to_string());
    //     }

    //     // Delete the secret if it's empty:
    //     if edited_secrets.get(&key.to_string()).is_none() {
    //         secrets.remove(&key.to_string());
    //     }
    // }

    // // Update into as a ref:
    // let secrets_to_update_refs: HashMap<&str, &str> = secrets
    //     .iter()
    //     .map(|(k, v)| (k.as_str(), v.as_str()))
    //     .collect();

    // vault
    //     .update_secret(&api_token, path, &secrets_to_update_refs)
    //     .await?;

    Ok(true)
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

fn generate_checklist_items(
    secrets: &HashMap<String, String>,
    edited_secrets: &HashMap<String, String>,
) -> Vec<(String, String)> {
    let max_key_len = edited_secrets
        .values()
        .map(|key| key.len())
        .max()
        .unwrap_or(0);
    let max_old_value_len = secrets.values().map(|key| key.len()).max().unwrap_or(0);

    let green = Style::new().for_stderr().green();
    let red = Style::new().for_stderr().red();

    let mut items: Vec<(String, String)> = edited_secrets
        .iter()
        .map(|(key, new_value)| {
            let mut key_with_style: String = key.clone();
            let mut old_value_with_style: String = new_value.clone();
            let mut new_value_with_style: String = new_value.clone();

            // Insert Key:
            if !secrets.contains_key(key) {
                key_with_style = format!("{}{}", green.apply_to("+"), green.apply_to(key));
            } else {
                // Updated values:
                let old_value = secrets.get(key).unwrap_or(&String::from("")).to_string();
                let changeset = Changeset::new(&old_value, new_value, "");

                let changes: Vec<String> = changeset
                    .diffs
                    .iter()
                    .map(|diff| match diff {
                        Difference::Same(s) => s.to_string(),
                        Difference::Add(s) => format!("{}", green.apply_to(s)),
                        Difference::Rem(s) => format!("{}", red.apply_to(s)),
                    })
                    .collect();

                new_value_with_style = changes.join("");
                old_value_with_style = old_value;
            }

            // Return the item:
            (
                format!(
                    "{:<width$} \t {} → {}",
                    key_with_style,
                    format_args!(
                        "{:<width$}",
                        old_value_with_style,
                        width = max_old_value_len
                    ),
                    new_value_with_style,
                    width = max_key_len
                ),
                key.to_owned(),
            )
        })
        .collect();

    // Handle deleted keys:
    let deleted_keys = secrets
        .keys()
        .filter(|key| !edited_secrets.contains_key(*key))
        .collect::<Vec<&String>>();

    for key in deleted_keys {
        let item = (
            format!(
                "{:<width$} \t {} → {}",
                format!("{}{}", style("-").red(), style(key).red()),
                format_args!(
                    "{:<width$}",
                    secrets.get(key).unwrap_or(&String::from("")),
                    width = max_old_value_len
                ),
                secrets.get(key).unwrap_or(&String::from("")),
                width = max_key_len
            ),
            key.to_owned(),
        );

        items.push(item);
    }

    items
}
