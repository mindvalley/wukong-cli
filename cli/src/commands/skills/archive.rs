use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crossterm::style::Stylize;

use super::common::{ensure_archive_readme, SKILLS_ARCHIVE_DIR, SKILLS_DIR};
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

enum ArchiveOutcome {
    Archived,
    AlreadyArchived,
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

/// Move an active skill into the archive folder and drop its `.claude` link.
/// Returns `AlreadyArchived` (without touching the active copy) if the slug is
/// already present in the archive, so re-runs never clobber archived skills.
fn archive_skill(root: &Path, slug: &str, agents_path: &Path) -> std::io::Result<ArchiveOutcome> {
    let archive_root = root.join(".agents").join(SKILLS_ARCHIVE_DIR);
    let archive_dir = archive_root.join(slug);

    if archive_dir.exists() {
        return Ok(ArchiveOutcome::AlreadyArchived);
    }

    fs::create_dir_all(&archive_root)?;
    ensure_archive_readme(&archive_root)?;
    fs::rename(agents_path, &archive_dir)?;
    remove_claude_link(root, slug)?;
    Ok(ArchiveOutcome::Archived)
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
        match archive_skill(&entry.root, &entry.slug, &entry.agents_path) {
            Ok(ArchiveOutcome::Archived) => {
                println!(
                    "  {} {}",
                    "Archived".green().bold(),
                    entry.label.clone().blue()
                );
            }
            Ok(ArchiveOutcome::AlreadyArchived) => {
                println!(
                    "  {} {} — already archived",
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

    fn write_active_skill(root: &Path, slug: &str) {
        let agents = root.join(".agents").join("skills").join(slug);
        fs::create_dir_all(&agents).unwrap();
        fs::write(agents.join("SKILL.md"), "# skill\n").unwrap();
        let claude = root.join(".claude").join("skills").join(slug);
        fs::create_dir_all(&claude).unwrap();
        symlink(
            format!("../../../.agents/skills/{slug}/SKILL.md"),
            claude.join("SKILL.md"),
        )
        .unwrap();
    }

    #[test]
    fn archive_moves_skill_removes_link_and_writes_readme() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let root = tmp.path();
        write_active_skill(root, "foo");
        let agents_path = root.join(".agents/skills/foo");

        let outcome = archive_skill(root, "foo", &agents_path).unwrap();
        assert!(matches!(outcome, ArchiveOutcome::Archived));

        assert!(root.join(".agents/skills-archive/foo/SKILL.md").is_file());
        assert!(!agents_path.exists());
        assert!(root
            .join(".claude/skills/foo/SKILL.md")
            .symlink_metadata()
            .is_err());
        assert!(root.join(".agents/skills-archive/README.md").is_file());
    }

    #[test]
    fn archive_skips_and_preserves_active_when_already_archived() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let root = tmp.path();
        write_active_skill(root, "foo");
        let archived = root.join(".agents/skills-archive/foo");
        fs::create_dir_all(&archived).unwrap();
        fs::write(archived.join("SKILL.md"), "# old\n").unwrap();
        let agents_path = root.join(".agents/skills/foo");

        let outcome = archive_skill(root, "foo", &agents_path).unwrap();
        assert!(matches!(outcome, ArchiveOutcome::AlreadyArchived));
        assert!(agents_path.join("SKILL.md").is_file());
        assert_eq!(
            fs::read_to_string(archived.join("SKILL.md")).unwrap(),
            "# old\n"
        );
    }
}
