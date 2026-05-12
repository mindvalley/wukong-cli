use std::{env, fs, path::{Path, PathBuf}};

use crossterm::style::Stylize;

use crate::{
    commands::Context, config::Config, error::WKCliError, loader::new_spinner,
    wukong_client::WKClient,
};

use wukong_sdk::graphql::check_skill_updates::SkillUpdateState;

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

const MANIFEST_FILE: &str = "mv-manifest.json";

#[derive(Debug)]
struct ManifestSkill {
    slug: String,
    hash: String,
    root: PathBuf,
    scope_label: String,
}

#[wukong_telemetry(command_event = "skills_update")]
pub async fn handle_skills_update(context: Context) -> Result<bool, WKCliError> {
    let installed = discover_manifest_skills();

    if installed.is_empty() {
        println!(
            "{}",
            "No registry-installed skills to check. Use `wukong skills add` to install skills."
                .yellow()
        );
        return Ok(true);
    }

    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let inputs: Vec<wukong_sdk::graphql::check_skill_updates::SkillUpdateCheckInput> = installed
        .iter()
        .map(
            |s| wukong_sdk::graphql::check_skill_updates::SkillUpdateCheckInput {
                slug: s.slug.clone(),
                current_hash: s.hash.clone(),
            },
        )
        .collect();

    let loader = new_spinner();
    loader.set_message("Checking for skill updates...");

    let response = wk_client.check_skill_updates(inputs).await?;
    loader.finish_and_clear();

    let mut updated: Vec<String> = Vec::new();
    let mut up_to_date: Vec<String> = Vec::new();
    let mut not_found: Vec<String> = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();

    for status in &response.check_skill_updates {
        let skill = match installed.iter().find(|s| s.slug == status.slug) {
            Some(s) => s,
            None => continue,
        };

        match status.status {
            SkillUpdateState::UP_TO_DATE => {
                up_to_date.push(status.slug.clone());
            }
            SkillUpdateState::UPDATE_AVAILABLE => {
                let loader = new_spinner();
                loader.set_message(format!("Updating {}...", status.slug));

                let skill_result = wk_client.fetch_skill(&status.slug).await;
                loader.finish_and_clear();

                match skill_result {
                    Ok(data) => {
                        let agents_dir =
                            skill.root.join(".agents").join("skills").join(&status.slug);
                        let agents_file = agents_dir.join("SKILL.md");
                        let claude_dir =
                            skill.root.join(".claude").join("skills").join(&status.slug);
                        let claude_file = claude_dir.join("SKILL.md");

                        fs::create_dir_all(&agents_dir)?;
                        fs::create_dir_all(&claude_dir)?;
                        fs::write(&agents_file, &data.skill.content)?;

                        if claude_file.exists() || claude_file.symlink_metadata().is_ok() {
                            let _ = fs::remove_file(&claude_file);
                            let relative_target: PathBuf = PathBuf::from("../../../.agents/skills")
                                .join(&status.slug)
                                .join("SKILL.md");

                            #[cfg(unix)]
                            let _ = std::os::unix::fs::symlink(&relative_target, &claude_file);
                        }

                        update_manifest(&skill.root, &status.slug, &data.skill.content_hash)?;

                        println!(
                            "  {} {} {}",
                            "Updated".green().bold(),
                            format!("[{}]", skill.scope_label).dark_grey(),
                            status.slug.clone().blue()
                        );
                        updated.push(status.slug.clone());
                    }
                    Err(err) => {
                        println!(
                            "  {} {} — {}",
                            "Failed".red().bold(),
                            status.slug.clone().blue(),
                            err
                        );
                        failed.push((status.slug.clone(), err.to_string()));
                    }
                }
            }
            SkillUpdateState::NOT_FOUND => {
                not_found.push(status.slug.clone());
            }
            _ => {}
        }
    }

    println!();
    println!(
        "{} skill(s) updated",
        updated.len().to_string().green().bold()
    );
    if !up_to_date.is_empty() {
        println!("  Up to date: {}", up_to_date.join(", "));
    }
    if !not_found.is_empty() {
        println!(
            "  {} {}",
            "Not found in registry:".yellow().bold(),
            not_found.join(", ")
        );
    }
    if !failed.is_empty() {
        for (slug, err) in &failed {
            println!("  {} {}", format!("{}:", slug).red(), err);
        }
    }

    Ok(true)
}

fn discover_manifest_skills() -> Vec<ManifestSkill> {
    let mut skills = Vec::new();

    if let Ok(cwd) = env::current_dir() {
        if let Some(entries) = read_manifest(&cwd) {
            for (slug, hash) in entries {
                skills.push(ManifestSkill {
                    slug,
                    hash,
                    root: cwd.clone(),
                    scope_label: "project".to_string(),
                });
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        if let Some(entries) = read_manifest(&home) {
            for (slug, hash) in entries {
                if !skills.iter().any(|s| s.slug == slug) {
                    skills.push(ManifestSkill {
                        slug,
                        hash,
                        root: home.clone(),
                        scope_label: "global".to_string(),
                    });
                }
            }
        }
    }

    skills
}

fn read_manifest(root: &Path) -> Option<std::collections::HashMap<String, String>> {
    let manifest_path = root.join(".agents").join("skills").join(MANIFEST_FILE);
    if !manifest_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&manifest_path).ok()?;
    serde_json::from_str(&content).ok()
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
