use std::{collections::HashMap, env, path::PathBuf};

use crossterm::style::Stylize;
use tabled::{style::Style, Table, Tabled};

use crate::{
    commands::Context, config::Config, error::WKCliError, loader::new_spinner,
    wukong_client::WKClient,
};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[derive(Debug, Clone)]
pub enum InstallScope {
    Project,
    Global,
    LocalOnly,
}

impl std::fmt::Display for InstallScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallScope::Project => write!(f, "✓ project"),
            InstallScope::Global => write!(f, "✓ global"),
            InstallScope::LocalOnly => write!(f, "local"),
        }
    }
}

#[derive(Tabled)]
pub struct SkillRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Slug")]
    pub slug: String,
    #[tabled(rename = "Description")]
    pub description: String,
    #[tabled(rename = "Installed")]
    pub installed: String,
}

#[wukong_telemetry(command_event = "skills_list")]
pub async fn handle_skills_list(context: Context) -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let loader = new_spinner();
    loader.set_message("Fetching skills from registry...");

    let result = wk_client.fetch_skills(None).await;
    loader.finish_and_clear();

    let response = result?;

    let registry_skills = &response.skills;

    let local_installed = discover_installed_skills();
    let mut local_slugs: Vec<String> = local_installed.keys().cloned().collect();
    local_slugs.sort();

    let mut rows: Vec<SkillRow> = Vec::new();

    for skill in registry_skills {
        let installed_label = match local_installed.get(&skill.slug) {
            Some(scope) => scope.to_string(),
            None => "—".to_string(),
        };
        rows.push(SkillRow {
            name: skill.name.clone(),
            slug: skill.slug.clone(),
            description: skill.description.clone(),
            installed: installed_label,
        });
    }

    let registry_slugs: Vec<String> = registry_skills.iter().map(|s| s.slug.clone()).collect();

    if !rows.is_empty() {
        let table = Table::new(&rows).with(Style::modern()).to_string();
        println!("{table}");
    } else {
        println!("{}", "No skills found in the registry.".yellow());
    }

    let local_only_slugs: Vec<String> = local_slugs
        .into_iter()
        .filter(|s| !registry_slugs.contains(s))
        .collect();

    if !local_only_slugs.is_empty() {
        println!();
        println!("{}", "Local skills (not from registry):".bold());
        let local_rows: Vec<SkillRow> = local_only_slugs
            .iter()
            .map(|slug| {
                let name = read_skill_name_from_file(slug);
                SkillRow {
                    name,
                    slug: slug.clone(),
                    description: String::new(),
                    installed: InstallScope::LocalOnly.to_string(),
                }
            })
            .collect();
        if !local_rows.is_empty() {
            let table = Table::new(&local_rows).with(Style::modern()).to_string();
            println!("{table}");
        }
    }

    Ok(true)
}

pub fn discover_installed_skills() -> HashMap<String, InstallScope> {
    let mut result = HashMap::new();

    if let Ok(cwd) = env::current_dir() {
        scan_skills_dir(
            &cwd.join(".agents").join("skills"),
            InstallScope::Project,
            &mut result,
        );
    }

    if let Some(home) = dirs::home_dir() {
        scan_skills_dir(
            &home.join(".agents").join("skills"),
            InstallScope::Global,
            &mut result,
        );
    }

    result
}

fn scan_skills_dir(dir: &PathBuf, scope: InstallScope, result: &mut HashMap<String, InstallScope>) {
    if !dir.is_dir() {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name != "mv-manifest.json" {
                    result
                        .entry(name.to_string())
                        .or_insert_with(|| scope.clone());
                }
            }
        }
    }
}

fn read_skill_name_from_file(slug: &str) -> String {
    let paths: Vec<PathBuf> = {
        let mut v = Vec::new();
        if let Ok(cwd) = env::current_dir() {
            v.push(
                cwd.join(".agents")
                    .join("skills")
                    .join(slug)
                    .join("SKILL.md"),
            );
        }
        if let Some(home) = dirs::home_dir() {
            v.push(
                home.join(".agents")
                    .join("skills")
                    .join(slug)
                    .join("SKILL.md"),
            );
        }
        v
    };

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(name) = parse_frontmatter_name(&content) {
                return name;
            }
        }
    }

    slug.to_string()
}

fn parse_frontmatter_name(content: &str) -> Option<String> {
    if !content.starts_with("---\n") {
        return None;
    }
    let rest = &content[4..];
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];
    for line in frontmatter.lines() {
        if let Some(value) = line.strip_prefix("name:") {
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value);
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}
