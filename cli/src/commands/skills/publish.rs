use std::{env, fs, path::PathBuf};

use crossterm::style::Stylize;
use inquire::{required, validator::Validation};
use regex::Regex;

use crate::{
    commands::Context, config::Config, error::WKCliError, loader::new_spinner,
    utils::inquire::inquire_render_config, wukong_client::WKClient,
};

use wukong_sdk::error::{APIError, WKError};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

const SLUG_REGEX: &str = r"^[a-z0-9][a-z0-9_-]{0,63}$";

#[derive(Debug, Clone)]
struct LocalSkill {
    label: String,
    folder_name: String,
    skill_md_path: PathBuf,
}

impl std::fmt::Display for LocalSkill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[wukong_telemetry(command_event = "skills_publish")]
pub async fn handle_skills_publish(context: Context) -> Result<bool, WKCliError> {
    let skills = discover_local_skills();

    if skills.is_empty() {
        println!(
            "{}",
            "No local skills found under .agents/skills/. Run `wukong skills init` first.".yellow()
        );
        return Ok(true);
    }

    let chosen = if skills.len() == 1 {
        let only = &skills[0];
        let confirm = inquire::Confirm::new(&format!("Publish {} ?", only.label))
            .with_render_config(inquire_render_config())
            .with_default(true)
            .prompt()?;
        if !confirm {
            println!("Aborted.");
            return Ok(false);
        }
        only.clone()
    } else {
        inquire::Select::new("Select a skill to publish", skills.clone())
            .with_render_config(inquire_render_config())
            .prompt()?
    };

    let content = fs::read_to_string(&chosen.skill_md_path)?;

    if !content.starts_with("---\n") || !content.contains("\n---") {
        println!(
            "{} {}",
            "SKILL.md is missing valid YAML frontmatter at".red(),
            chosen.skill_md_path.display().to_string().blue()
        );
        return Ok(false);
    }

    let slug_re = Regex::new(SLUG_REGEX).unwrap();
    let default_slug = chosen.folder_name.clone();
    let slug = inquire::Text::new("Slug")
        .with_render_config(inquire_render_config())
        .with_default(&default_slug)
        .with_validator(required!("Slug is required"))
        .with_validator(move |input: &str| {
            if slug_re.is_match(input) {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    "must match ^[a-z0-9][a-z0-9_-]{0,63}$ (lowercase alphanumeric, underscore, dash; up to 64 chars; must start with letter or digit)"
                        .into(),
                ))
            }
        })
        .with_help_message("Used as the folder name in the registry repo (skills/<slug>/SKILL.md)")
        .prompt()?
        .trim()
        .to_string();

    let raw_msg = inquire::Text::new("Commit message (optional)")
        .with_render_config(inquire_render_config())
        .with_help_message("Leave blank to use the server default")
        .prompt_skippable()?
        .unwrap_or_default();
    let commit_message = if raw_msg.trim().is_empty() {
        None
    } else {
        Some(raw_msg)
    };

    println!();
    println!("About to publish:");
    println!(
        "  source     {}",
        chosen.skill_md_path.display().to_string().blue()
    );
    println!("  slug       {}", slug.clone().bold());
    println!("  bytes      {}", content.len());
    if let Some(msg) = &commit_message {
        println!("  message    {}", msg.clone().dark_grey());
    }
    println!();

    let confirm = inquire::Confirm::new("Publish to skills registry?")
        .with_render_config(inquire_render_config())
        .with_default(true)
        .prompt()?;
    if !confirm {
        println!("Aborted.");
        return Ok(false);
    }

    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let loader = new_spinner();
    loader.set_message("Publishing skill...");

    let result = wk_client
        .publish_skill(&slug, &content, commit_message.clone())
        .await;

    loader.finish_and_clear();

    match result {
        Ok(data) => {
            let pub_result = data.publish_skill;
            let short_sha: String = pub_result.commit_sha.chars().take(7).collect();
            println!();
            println!("{}", "Published.".green().bold());
            println!("  path       {}", pub_result.path.blue());
            println!("  commit     {}", short_sha.dark_grey());
            println!("  pull req   {}", pub_result.html_url.blue());
            println!();
            println!(
                "{}",
                "Auto-merge is queued — the PR will land once required checks pass.".dark_grey()
            );
            Ok(true)
        }
        Err(err) => {
            print_friendly_error(&err);
            Err(err)
        }
    }
}

fn discover_local_skills() -> Vec<LocalSkill> {
    let mut roots: Vec<(&'static str, PathBuf)> = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        roots.push(("project", cwd.join(".agents").join("skills")));
    }
    if let Some(home) = dirs::home_dir() {
        roots.push(("global", home.join(".agents").join("skills")));
    }

    let mut entries = Vec::new();
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
            let skill_md = path.join("SKILL.md");
            if !skill_md.is_file() {
                continue;
            }
            let folder_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            entries.push(LocalSkill {
                label: format!("[{}] {}", scope, folder_name),
                folder_name,
                skill_md_path: skill_md,
            });
        }
    }
    entries
}

fn print_friendly_error(err: &WKCliError) {
    if let WKCliError::WKSdkError(WKError::APIError(APIError::ResponseError { message, .. })) = err
    {
        if message.contains("not authorized to publish skills") {
            println!(
                "{}",
                "Your account is not on the skills registry allowlist."
                    .red()
                    .bold()
            );
            println!(
                "{}",
                "Ask infra to add your email to :skills_registry.allowed_publishers.".dark_grey()
            );
            return;
        }
        if message.starts_with("invalid slug") {
            println!("{} {}", "Slug rejected by server:".red(), message);
            return;
        }
        if message.starts_with("invalid SKILL.md") {
            println!("{} {}", "SKILL.md rejected by server:".red(), message);
            return;
        }
        if message.starts_with("invalid commit_message") || message.starts_with("commit_message ") {
            println!("{} {}", "Commit message rejected:".red(), message);
            return;
        }
        if message.contains("skills registry publishers not configured") {
            println!(
                "{}",
                "The server has no allowed publishers configured for the skills registry.".red()
            );
            return;
        }
    }
    println!("{} {}", "Publish failed:".red(), err);
}
