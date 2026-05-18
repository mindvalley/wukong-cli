use std::{env, fs, path::PathBuf};

use crossterm::style::Stylize;

use crate::{commands::Context, error::WKCliError, utils::inquire::inquire_render_config};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[derive(Debug, Clone)]
struct SkillEntry {
    label: String,
    path: PathBuf,
}

impl std::fmt::Display for SkillEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[wukong_telemetry(command_event = "skills_remove")]
pub async fn handle_skills_remove(context: Context) -> Result<bool, WKCliError> {
    let cwd = env::current_dir()?;
    let home = dirs::home_dir();

    let mut roots: Vec<(String, PathBuf)> = Vec::new();
    if let Some(h) = home.as_ref() {
        roots.push((
            "global:.claude".to_string(),
            h.join(".claude").join("skills"),
        ));
        roots.push((
            "global:.agents".to_string(),
            h.join(".agents").join("skills"),
        ));
    }
    roots.push((
        "project:.claude".to_string(),
        cwd.join(".claude").join("skills"),
    ));
    roots.push((
        "project:.agents".to_string(),
        cwd.join(".agents").join("skills"),
    ));

    let mut entries: Vec<SkillEntry> = Vec::new();
    for (scope, root) in &roots {
        if !root.is_dir() {
            continue;
        }
        let read = match fs::read_dir(root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for dir_entry in read.flatten() {
            let path = dir_entry.path();
            if !path.is_dir() {
                continue;
            }
            if !path.join("SKILL.md").is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<unknown>")
                .to_string();
            entries.push(SkillEntry {
                label: format!("[{}] {}", scope, name),
                path,
            });
        }
    }

    if entries.is_empty() {
        println!("No installed skills found.");
        return Ok(true);
    }

    let selected = inquire::MultiSelect::new("Select skills to remove", entries.clone())
        .with_render_config(inquire_render_config())
        .with_help_message("↑↓ to move, space to select, ↵ to confirm, esc to cancel")
        .prompt()?;

    if selected.is_empty() {
        println!("No skills selected.");
        return Ok(true);
    }

    let confirm = inquire::Confirm::new(&format!(
        "Delete {} skill(s)? This cannot be undone.",
        selected.len()
    ))
    .with_render_config(inquire_render_config())
    .with_default(false)
    .prompt()?;

    if !confirm {
        println!("Aborted.");
        return Ok(false);
    }

    let mut failures: Vec<(String, std::io::Error)> = Vec::new();
    for entry in &selected {
        match fs::remove_dir_all(&entry.path) {
            Ok(_) => {
                println!(
                    "  {} {}",
                    "Removed".green().bold(),
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
                failures.push((entry.label.clone(), err));
            }
        }
    }

    Ok(failures.is_empty())
}
