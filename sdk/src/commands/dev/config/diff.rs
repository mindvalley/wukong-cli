use crate::error::{DevConfigError, WKError};
use crate::loader::new_spinner_progress_bar;
use crate::services::vault::Vault;
use crate::utils::line::Line;
use dialoguer::console::{style, Style};
use owo_colors::OwoColorize;
use similar::{ChangeTag, TextDiff};

use super::utils::{
    extract_secret_infos, get_local_config_path, get_secret_config_files, get_updated_configs,
    make_path_relative,
};

pub async fn handle_config_diff() -> Result<bool, WKError> {
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
        println!("The config file is already up to date with the Vault Bunker.");
        return Ok(true);
    }

    for (secret_annotation, remote_config, local_config, config_path) in &updated_configs {
        let local_config_path =
            get_local_config_path(config_path, &secret_annotation.destination_file);

        print_diff(
            remote_config,
            local_config,
            &local_config_path.to_string_lossy(),
        );
    }

    Ok(true)
}

pub fn has_diff(old_secret_config: &str, new_secret_config: &str) -> bool {
    let changeset = TextDiff::from_lines(old_secret_config, new_secret_config);

    changeset
        .iter_all_changes()
        .any(|change| matches!(change.tag(), ChangeTag::Delete | ChangeTag::Insert))
}

pub fn print_diff(old_secret_config: &str, new_secret_config: &str, local_config_path: &str) {
    println!();
    println!("{}", make_path_relative(local_config_path).dimmed());

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_has_diff() {
        let old_secret_config = "first line\nsecond line\nthird line";
        let new_secret_config = "first line\nnew second line\nthird line";

        assert_eq!(has_diff(old_secret_config, new_secret_config), true);

        let old_secret_config = "first line\nsecond line\nthird line";
        let new_secret_config = "first line\nsecond line\nthird line";

        assert_eq!(has_diff(old_secret_config, new_secret_config), false);

        let old_secret_config = "first line\nsecond line";
        let new_secret_config = "first line\nsecond line\nthird line";

        assert_eq!(has_diff(old_secret_config, new_secret_config), true);

        let old_secret_config = "first line\nsecond line\nthird line";
        let new_secret_config = "first line\nsecond line";

        assert_eq!(has_diff(old_secret_config, new_secret_config), true);
    }
}