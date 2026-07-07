use std::{env, fs, path::PathBuf};

use crossterm::style::Stylize;

use super::common::{create_skill_symlink, SKILLS_ARCHIVE_DIR, SKILLS_DIR};
use crate::{commands::Context, error::WKCliError, utils::inquire::inquire_render_config};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[derive(Debug, Clone)]
struct RestoreEntry {
    label: String,
    slug: String,
    root: PathBuf,
    archive_path: PathBuf,
}

impl std::fmt::Display for RestoreEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[wukong_telemetry(command_event = "skills_restore")]
pub async fn handle_skills_restore(context: Context) -> Result<bool, WKCliError> {
    let cwd = env::current_dir()?;
    let home = dirs::home_dir();

    let mut roots: Vec<(String, PathBuf)> = Vec::new();
    if let Some(h) = home.as_ref() {
        roots.push(("global".to_string(), h.clone()));
    }
    roots.push(("project".to_string(), cwd));

    let mut entries: Vec<RestoreEntry> = Vec::new();
    for (scope, root) in &roots {
        let archive_dir = root.join(".agents").join(SKILLS_ARCHIVE_DIR);
        if !archive_dir.is_dir() {
            continue;
        }
        let read = match fs::read_dir(&archive_dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for dir_entry in read.flatten() {
            let path = dir_entry.path();
            if !path.is_dir() || !path.join("SKILL.md").is_file() {
                continue;
            }
            let slug = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            entries.push(RestoreEntry {
                label: format!("[{}] {}", scope, slug),
                slug,
                root: root.clone(),
                archive_path: path,
            });
        }
    }

    if entries.is_empty() {
        println!("No archived skills to restore.");
        return Ok(true);
    }

    let selected = inquire::MultiSelect::new("Select skills to restore", entries.clone())
        .with_render_config(inquire_render_config())
        .with_help_message("↑↓ to move, space to select, ↵ to confirm, esc to cancel")
        .prompt()?;

    if selected.is_empty() {
        println!("No skills selected.");
        return Ok(true);
    }

    let mut failures: Vec<String> = Vec::new();
    for entry in &selected {
        let active_root = entry.root.join(".agents").join(SKILLS_DIR);
        let active_dir = active_root.join(&entry.slug);

        if active_dir.exists() {
            println!(
                "  {} {} — already active",
                "Skipping".yellow().bold(),
                entry.label.clone().blue()
            );
            continue;
        }

        let result = (|| -> Result<(), WKCliError> {
            fs::create_dir_all(&active_root)?;
            fs::rename(&entry.archive_path, &active_dir)?;

            let agents_file = active_dir.join("SKILL.md");
            let claude_dir = entry
                .root
                .join(".claude")
                .join(SKILLS_DIR)
                .join(&entry.slug);
            let claude_file = claude_dir.join("SKILL.md");
            fs::create_dir_all(&claude_dir)?;
            create_skill_symlink(&agents_file, &claude_file)?;
            Ok(())
        })();

        match result {
            Ok(_) => {
                println!(
                    "  {} {}",
                    "Restored".green().bold(),
                    entry.label.clone().blue()
                );
            }
            Err(err) => {
                println!(
                    "  {} {}: {}",
                    "Failed".red().bold(),
                    entry.label.clone().blue(),
                    err
                );
                failures.push(entry.label.clone());
            }
        }
    }

    Ok(failures.is_empty())
}
