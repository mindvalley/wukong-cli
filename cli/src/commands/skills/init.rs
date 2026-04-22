use std::{env, fs, path::PathBuf};

use crossterm::style::Stylize;
use inquire::required;

use crate::{commands::Context, error::WKCliError, utils::inquire::inquire_render_config};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[wukong_telemetry(command_event = "skills_init")]
pub async fn handle_skills_init(context: Context) -> Result<bool, WKCliError> {
    let cwd = env::current_dir()?;

    let skill_name = inquire::Text::new("Skill name")
        .with_render_config(inquire_render_config())
        .with_validator(required!("Skill name is required"))
        .with_placeholder("my-skill")
        .with_help_message("Used as the folder name under ./.claude/skills/")
        .prompt()?
        .trim()
        .to_string();

    if skill_name.is_empty() {
        println!("{}", "Skill name is required.".red());
        return Ok(false);
    }

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

    let confirm = inquire::Confirm::new(&format!("Create new skill at {} ?", skill_file.display()))
        .with_render_config(inquire_render_config())
        .with_default(true)
        .prompt()?;

    if !confirm {
        println!("Aborted.");
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
