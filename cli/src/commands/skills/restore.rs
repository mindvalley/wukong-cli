use std::{
    env, fs,
    path::{Path, PathBuf},
};

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

enum RestoreOutcome {
    Restored,
    AlreadyActive,
}

/// Move an archived skill back into the active folder and recreate its
/// `.claude` symlink. Returns `AlreadyActive` (without touching the archive) if
/// the slug is already active. The symlink is recreated idempotently: any stale
/// link/file left behind by a previous partial run is removed first, so this
/// can't fail with `EEXIST` and leave a half-restored, misreported state.
fn restore_skill(
    root: &Path,
    slug: &str,
    archive_path: &Path,
) -> Result<RestoreOutcome, WKCliError> {
    let active_root = root.join(".agents").join(SKILLS_DIR);
    let active_dir = active_root.join(slug);

    if active_dir.exists() {
        return Ok(RestoreOutcome::AlreadyActive);
    }

    fs::create_dir_all(&active_root)?;
    fs::rename(archive_path, &active_dir)?;

    let agents_file = active_dir.join("SKILL.md");
    let claude_dir = root.join(".claude").join(SKILLS_DIR).join(slug);
    let claude_file = claude_dir.join("SKILL.md");
    fs::create_dir_all(&claude_dir)?;
    if claude_file.symlink_metadata().is_ok() {
        fs::remove_file(&claude_file)?;
    }
    create_skill_symlink(&agents_file, &claude_file)?;
    Ok(RestoreOutcome::Restored)
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
        match restore_skill(&entry.root, &entry.slug, &entry.archive_path) {
            Ok(RestoreOutcome::Restored) => {
                println!(
                    "  {} {}",
                    "Restored".green().bold(),
                    entry.label.clone().blue()
                );
            }
            Ok(RestoreOutcome::AlreadyActive) => {
                println!(
                    "  {} {} — already active",
                    "Skipping".yellow().bold(),
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn write_archived_skill(root: &Path, slug: &str) {
        let archived = root.join(".agents").join("skills-archive").join(slug);
        fs::create_dir_all(&archived).unwrap();
        fs::write(archived.join("SKILL.md"), "# skill\n").unwrap();
    }

    #[test]
    fn restore_moves_back_and_recreates_symlink() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let root = tmp.path();
        write_archived_skill(root, "foo");
        let archive_path = root.join(".agents/skills-archive/foo");

        let outcome = restore_skill(root, "foo", &archive_path).unwrap();
        assert!(matches!(outcome, RestoreOutcome::Restored));

        assert!(root.join(".agents/skills/foo/SKILL.md").is_file());
        assert!(!archive_path.exists());
        let link = root.join(".claude/skills/foo/SKILL.md");
        assert!(link.symlink_metadata().unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&link).unwrap(), "# skill\n");
    }

    #[test]
    fn restore_is_idempotent_with_stale_claude_link() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let root = tmp.path();
        write_archived_skill(root, "foo");
        // stale leftover link (e.g. from a previous partially-failed archive)
        let claude = root.join(".claude/skills/foo");
        fs::create_dir_all(&claude).unwrap();
        symlink(
            "../../../.agents/skills/foo/SKILL.md",
            claude.join("SKILL.md"),
        )
        .unwrap();
        let archive_path = root.join(".agents/skills-archive/foo");

        let outcome = restore_skill(root, "foo", &archive_path).unwrap();
        assert!(matches!(outcome, RestoreOutcome::Restored));
        let link = claude.join("SKILL.md");
        assert!(link.symlink_metadata().unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&link).unwrap(), "# skill\n");
    }

    #[test]
    fn restore_skips_and_preserves_active_when_already_active() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let root = tmp.path();
        write_archived_skill(root, "foo");
        let active = root.join(".agents/skills/foo");
        fs::create_dir_all(&active).unwrap();
        fs::write(active.join("SKILL.md"), "# active\n").unwrap();
        let archive_path = root.join(".agents/skills-archive/foo");

        let outcome = restore_skill(root, "foo", &archive_path).unwrap();
        assert!(matches!(outcome, RestoreOutcome::AlreadyActive));
        assert!(archive_path.join("SKILL.md").is_file());
        assert_eq!(
            fs::read_to_string(active.join("SKILL.md")).unwrap(),
            "# active\n"
        );
    }
}
