use std::{env, fs, path::PathBuf};

use crossterm::style::Stylize;

use crate::{commands::Context, error::WKCliError};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[wukong_telemetry(command_event = "skills_init")]
pub async fn handle_skills_init(
    context: Context,
    name: Option<String>,
) -> Result<bool, WKCliError> {
    let cwd = env::current_dir()?;

    let skill_name = match name {
        Some(n) if !n.trim().is_empty() => n,
        _ => cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("skill")
            .to_string(),
    };

    let skill_dir: PathBuf = cwd.join(".claude").join("skills").join(&skill_name);
    let skill_file = skill_dir.join("SKILL.md");

    if skill_file.exists() {
        println!(
            "{} {}",
            "Skill already exists at".yellow(),
            skill_file.display().to_string().blue()
        );
        return Ok(false);
    }

    fs::create_dir_all(&skill_dir)?;

    let template = format!(
        "---\nname: {name}\ndescription: A brief description of what this skill does\n---\n\n# {name}\n\nInstructions for the agent to follow when this skill is activated.\n\n## When to use\n\nDescribe when this skill should be used.\n\n## Instructions\n\n1. First step\n2. Second step\n3. Additional steps as needed\n",
        name = skill_name
    );
    fs::write(&skill_file, template)?;

    println!(
        "  {} skill at {}",
        "Created".green().bold(),
        skill_file.display().to_string().blue()
    );
    println!();
    println!("Next steps:");
    println!(
        "  1. Edit {} to define your skill instructions",
        skill_file.display().to_string().blue()
    );
    println!("  2. Update the name and description in the frontmatter");

    Ok(true)
}
