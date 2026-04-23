use std::{env, fs, path::PathBuf};

use crossterm::style::Stylize;
use inquire::required;

use crate::{commands::Context, error::WKCliError, utils::inquire::inquire_render_config};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Project,
    Global,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Project => write!(f, "Project (current directory)"),
            Scope::Global => write!(f, "Global (home directory)"),
        }
    }
}

#[wukong_telemetry(command_event = "skills_init")]
pub async fn handle_skills_init(context: Context) -> Result<bool, WKCliError> {
    if !cfg!(unix) {
        println!(
            "{}",
            "wukong skills init is only supported on Unix-like systems (uses symlinks).".red()
        );
        return Ok(false);
    }

    let scope = inquire::Select::new(
        "Where should this skill live?",
        vec![Scope::Project, Scope::Global],
    )
    .with_render_config(inquire_render_config())
    .prompt()?;

    let root: PathBuf = match scope {
        Scope::Project => env::current_dir()?,
        Scope::Global => match dirs::home_dir() {
            Some(h) => h,
            None => {
                println!("{}", "Unable to locate home directory.".red());
                return Ok(false);
            }
        },
    };

    let skill_name = inquire::Text::new("Skill name")
        .with_render_config(inquire_render_config())
        .with_validator(required!("Skill name is required"))
        .with_placeholder("my-skill")
        .with_help_message("Used as the folder name under .agents/skills/ and .claude/skills/")
        .prompt()?
        .trim()
        .to_string();

    if skill_name.is_empty() {
        println!("{}", "Skill name is required.".red());
        return Ok(false);
    }

    let agents_dir = root.join(".agents").join("skills").join(&skill_name);
    let agents_file = agents_dir.join("SKILL.md");
    let claude_dir = root.join(".claude").join("skills").join(&skill_name);
    let claude_file = claude_dir.join("SKILL.md");

    if agents_file.exists() {
        println!(
            "{} {}",
            "Skill source already exists at".yellow(),
            agents_file.display().to_string().blue()
        );
        return Ok(false);
    }
    if claude_file.exists() || claude_file.symlink_metadata().is_ok() {
        println!(
            "{} {}",
            "Skill symlink already exists at".yellow(),
            claude_file.display().to_string().blue()
        );
        return Ok(false);
    }

    let confirm = inquire::Confirm::new(&format!(
        "Create skill at {} (symlinked from {}) ?",
        agents_file.display(),
        claude_file.display()
    ))
    .with_render_config(inquire_render_config())
    .with_default(true)
    .prompt()?;

    if !confirm {
        println!("Aborted.");
        return Ok(false);
    }

    fs::create_dir_all(&agents_dir)?;
    fs::create_dir_all(&claude_dir)?;

    let template = format!(
        "---\nname: {name}\ndescription: A brief description of what this skill does\n---\n\n# {name}\n\nInstructions for the agent to follow when this skill is activated.\n\n## When to use\n\nDescribe when this skill should be used.\n\n## Instructions\n\n1. First step\n2. Second step\n3. Additional steps as needed\n",
        name = skill_name
    );
    fs::write(&agents_file, template)?;

    // Symlink target is relative to the symlink's parent directory:
    // claude_dir = <root>/.claude/skills/<name>/
    // Up 3 levels reaches <root>, then into .agents/skills/<name>/SKILL.md.
    let relative_target: PathBuf = PathBuf::from("../../../.agents/skills")
        .join(&skill_name)
        .join("SKILL.md");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&relative_target, &claude_file)?;

    println!(
        "  {} skill at {}",
        "Created".green().bold(),
        agents_file.display().to_string().blue()
    );
    println!(
        "  {} symlink at {}",
        "Linked".green().bold(),
        claude_file.display().to_string().blue()
    );
    println!();
    println!("Next steps:");
    println!(
        "  1. Edit {} to define your skill instructions",
        agents_file.display().to_string().blue()
    );
    println!("  2. Update the name and description in the frontmatter");

    Ok(true)
}
