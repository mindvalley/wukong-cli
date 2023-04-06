use crate::error::{CliError, DevConfigError};
use crate::loader::new_spinner_progress_bar;
use crate::services::vault::Vault;
use crate::utils::line::Line;
use dialoguer::console::{style, Style};
use owo_colors::OwoColorize;
use similar::{ChangeTag, TextDiff};

use super::utils::{
    filter_config_with_secret_annotations, get_available_files, get_updated_configs,
    remove_parent_directories,
};

pub async fn handle_config_diff() -> Result<bool, CliError> {
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
        println!("The config file is already up to date with the Vault Bunker.");
        return Ok(true);
    }

    for (config_path, (_, remote_config, local_config)) in &updated_configs {
        print_diff(remote_config, local_config, config_path);
    }

    Ok(true)
}

pub fn has_diff(old_secret_config: &str, new_secret_config: &str) -> bool {
    let changeset = TextDiff::from_lines(old_secret_config, new_secret_config);

    changeset
        .iter_all_changes()
        .any(|change| matches!(change.tag(), ChangeTag::Delete | ChangeTag::Insert))
}

pub fn print_diff(old_secret_config: &str, new_secret_config: &str, secret_path: &str) {
    println!();
    println!("{}", remove_parent_directories(secret_path).dimmed());

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
