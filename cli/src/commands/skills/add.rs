use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crossterm::style::Stylize;
use inquire::{MultiSelect, Select};

use crate::{
    commands::Context, config::Config, error::WKCliError, loader::new_spinner,
    utils::inquire::inquire_render_config, wukong_client::WKClient,
};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

const MANIFEST_FILE: &str = "mv-manifest.json";

#[derive(Debug, Clone)]
struct SkillChoice {
    slug: String,
    name: String,
}

impl std::fmt::Display for SkillChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} — {}", self.slug, self.name)
    }
}

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

#[wukong_telemetry(command_event = "skills_add")]
pub async fn handle_skills_add(
    context: Context,
    name: &Option<String>,
    global: bool,
    project: bool,
) -> Result<bool, WKCliError> {
    if global && project {
        println!("{}", "Cannot specify both --global and --project.".red());
        return Ok(false);
    }

    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let loader = new_spinner();
    loader.set_message("Fetching skills from registry...");
    let response = wk_client.fetch_skills(name.as_deref()).await?;
    loader.finish_and_clear();

    let all_skills = &response.skills;

    if all_skills.is_empty() {
        if let Some(keyword) = name {
            println!(
                "{}",
                format!("No skills found matching '{}'.", keyword).yellow()
            );
        } else {
            println!("{}", "No skills available in the registry.".yellow());
        }
        return Ok(true);
    }

    let choices: Vec<SkillChoice> = all_skills
        .iter()
        .map(|s| SkillChoice {
            slug: s.slug.clone(),
            name: s.name.clone(),
        })
        .collect();

    let selected = if name.is_some() && choices.len() == 1 {
        let confirm = inquire::Confirm::new(&format!(
            "Found one skill: {} — {}. Install it?",
            choices[0].slug, choices[0].name
        ))
        .with_render_config(inquire_render_config())
        .with_default(true)
        .prompt()?;
        if !confirm {
            println!("Aborted.");
            return Ok(false);
        }
        vec![choices[0].clone()]
    } else {
        MultiSelect::new("Select skills to install:", choices.clone())
            .with_render_config(inquire_render_config())
            .with_help_message("↑↓ move, space select, ↵ confirm")
            .prompt()?
    };

    if selected.is_empty() {
        println!("No skills selected.");
        return Ok(true);
    }

    let scope = if global {
        Scope::Global
    } else if project {
        Scope::Project
    } else {
        Select::new(
            "Where should these skills be installed?",
            vec![Scope::Project, Scope::Global],
        )
        .with_render_config(inquire_render_config())
        .prompt()?
    };

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

    let scope_tag = format!(
        "[{}]",
        match scope {
            Scope::Project => "project",
            Scope::Global => "global",
        }
    );

    let mut installed_count = 0usize;
    let mut skipped_count = 0usize;

    for choice in &selected {
        let agents_dir = root.join(".agents").join("skills").join(&choice.slug);
        let claude_dir = root.join(".claude").join("skills").join(&choice.slug);
        let agents_file = agents_dir.join("SKILL.md");
        let claude_file = claude_dir.join("SKILL.md");

        if agents_file.exists() {
            println!(
                "  {} {} {} — already installed",
                "Skipping".yellow().bold(),
                scope_tag.clone().dark_grey(),
                choice.slug.clone().blue()
            );
            skipped_count += 1;
            continue;
        }

        let loader = new_spinner();
        loader.set_message(format!("Downloading {}...", choice.slug));

        let skill_result = wk_client.fetch_skill(&choice.slug).await;
        loader.finish_and_clear();

        let skill_data = match skill_result {
            Ok(data) => data,
            Err(err) => {
                println!(
                    "  {} {} — {}",
                    "Failed".red().bold(),
                    choice.slug.clone().blue(),
                    err
                );
                continue;
            }
        };

        fs::create_dir_all(&agents_dir)?;
        fs::create_dir_all(&claude_dir)?;
        fs::write(&agents_file, &skill_data.skill.content)?;

        let relative_target: PathBuf = PathBuf::from("../../../.agents/skills")
            .join(&choice.slug)
            .join("SKILL.md");

        #[cfg(unix)]
        std::os::unix::fs::symlink(&relative_target, &claude_file)?;

        update_manifest(&root, &choice.slug, &skill_data.skill.content_hash)?;

        println!(
            "  {} {} {}",
            "Installed".green().bold(),
            scope_tag.clone().dark_grey(),
            choice.slug.clone().blue()
        );
        installed_count += 1;
    }

    println!();
    if installed_count > 0 {
        println!(
            "{} {} skill(s){}.",
            "Installed".green().bold(),
            installed_count,
            if skipped_count > 0 {
                format!(", {} skipped", skipped_count)
            } else {
                String::new()
            }
        );
    } else if skipped_count > 0 {
        println!(
            "{} All {} skill(s) were already installed.",
            "Skipped:".yellow().bold(),
            skipped_count
        );
    }

    Ok(true)
}

fn update_manifest(root: &Path, slug: &str, content_hash: &str) -> Result<(), WKCliError> {
    let manifest_path = root.join(".agents").join("skills").join(MANIFEST_FILE);

    let mut manifest: std::collections::HashMap<String, String> = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    manifest.insert(slug.to_string(), content_hash.to_string());

    let json = serde_json::to_string_pretty(&manifest).map_err(|e| {
        WKCliError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    fs::write(&manifest_path, json)?;

    Ok(())
}
