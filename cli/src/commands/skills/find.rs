use crossterm::style::Stylize;
use tabled::{style::Style, Table};

use crate::{
    commands::Context, config::Config, error::WKCliError, loader::new_spinner,
    wukong_client::WKClient,
};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

use super::list::{discover_installed_skills, SkillRow};

#[wukong_telemetry(command_event = "skills_find")]
pub async fn handle_skills_find(context: Context, keyword: &str) -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let loader = new_spinner();
    loader.set_message(format!("Searching skills for '{}'...", keyword));

    let result = wk_client.fetch_skills(Some(keyword)).await;
    loader.finish_and_clear();

    let response = result?;

    let registry_skills = &response.skills;

    if registry_skills.is_empty() {
        println!(
            "{}",
            format!("No skills found matching '{}'.", keyword).yellow()
        );
        return Ok(true);
    }

    let local_installed = discover_installed_skills();

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

    let table = Table::new(&rows).with(Style::modern()).to_string();
    println!("{table}");

    Ok(true)
}
