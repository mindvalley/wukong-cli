use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crossterm::style::Stylize;

use super::common::{SKILLS_ARCHIVE_DIR, SKILLS_DIR};
use crate::{commands::Context, error::WKCliError, utils::inquire::inquire_render_config};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[derive(Debug, Clone)]
struct ArchiveEntry {
    label: String,
    slug: String,
    root: PathBuf,
    agents_path: PathBuf,
}

impl std::fmt::Display for ArchiveEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

fn remove_claude_link(root: &Path, slug: &str) -> std::io::Result<()> {
    let claude = root.join(".claude").join(SKILLS_DIR).join(slug);
    let meta = match claude.symlink_metadata() {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };
    if meta.is_dir() {
        fs::remove_dir_all(&claude)
    } else {
        fs::remove_file(&claude)
    }
}

#[wukong_telemetry(command_event = "skills_archive")]
pub async fn handle_skills_archive(context: Context) -> Result<bool, WKCliError> {
    let cwd = env::current_dir()?;
    let home = dirs::home_dir();

    let mut roots: Vec<(String, PathBuf)> = Vec::new();
    if let Some(h) = home.as_ref() {
        roots.push(("global".to_string(), h.clone()));
    }
    roots.push(("project".to_string(), cwd));

    let mut entries: Vec<ArchiveEntry> = Vec::new();
    for (scope, root) in &roots {
        let skills_dir = root.join(".agents").join(SKILLS_DIR);
        if !skills_dir.is_dir() {
            continue;
        }
        let read = match fs::read_dir(&skills_dir) {
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
            entries.push(ArchiveEntry {
                label: format!("[{}] {}", scope, slug),
                slug,
                root: root.clone(),
                agents_path: path,
            });
        }
    }

    if entries.is_empty() {
        println!("No active skills to archive.");
        return Ok(true);
    }

    let selected = inquire::MultiSelect::new("Select skills to archive", entries.clone())
        .with_render_config(inquire_render_config())
        .with_help_message("↑↓ to move, space to select, ↵ to confirm, esc to cancel")
        .prompt()?;

    if selected.is_empty() {
        println!("No skills selected.");
        return Ok(true);
    }

    let mut failures: Vec<String> = Vec::new();
    for entry in &selected {
        let archive_root = entry.root.join(".agents").join(SKILLS_ARCHIVE_DIR);
        let archive_dir = archive_root.join(&entry.slug);

        if archive_dir.exists() {
            println!(
                "  {} {} — already archived",
                "Skipping".yellow().bold(),
                entry.label.clone().blue()
            );
            continue;
        }

        let result = (|| -> std::io::Result<()> {
            fs::create_dir_all(&archive_root)?;
            fs::rename(&entry.agents_path, &archive_dir)?;
            remove_claude_link(&entry.root, &entry.slug)?;
            Ok(())
        })();

        match result {
            Ok(_) => {
                println!(
                    "  {} {}",
                    "Archived".green().bold(),
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
