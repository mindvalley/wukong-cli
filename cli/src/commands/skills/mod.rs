mod add;
mod find;
mod init;
mod list;
mod publish;
mod remove;
mod update;

use clap::{Args, Subcommand};

use crate::error::WKCliError;

use self::{
    add::handle_skills_add, find::handle_skills_find, init::handle_skills_init,
    list::handle_skills_list, publish::handle_skills_publish, remove::handle_skills_remove,
    update::handle_skills_update,
};

use super::Context;

#[derive(Debug, Args)]
pub struct Skills {
    #[command(subcommand)]
    pub subcommand: SkillsSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum SkillsSubcommand {
    /// Display all available skills from the registry
    List,
    /// Search for skills by keyword
    Find {
        /// Keyword to search for
        keyword: String,
    },
    /// Scaffold a new skill at ./.claude/skills/<name>/SKILL.md (interactive)
    Init,
    /// Publish a local skill to the internal skills registry repo
    Publish,
    /// Install one or more skills from the registry locally
    Add {
        /// Skill name or keyword to search for
        name: Option<String>,
        /// Install globally (home directory) instead of the current project
        #[arg(long)]
        global: bool,
        /// Install to the current project directory (overrides --global)
        #[arg(long)]
        project: bool,
    },
    /// Remove installed skills from local and global skill directories
    Remove,
    /// Update all outdated installed skills from the registry
    Update,
}

impl Skills {
    pub async fn handle_command(&self, context: Context) -> Result<bool, WKCliError> {
        match &self.subcommand {
            SkillsSubcommand::List => handle_skills_list(context).await,
            SkillsSubcommand::Find { keyword } => handle_skills_find(context, keyword).await,
            SkillsSubcommand::Init => handle_skills_init(context).await,
            SkillsSubcommand::Publish => handle_skills_publish(context).await,
            SkillsSubcommand::Add {
                name,
                global,
                project,
            } => handle_skills_add(context, name, *global, *project).await,
            SkillsSubcommand::Remove => handle_skills_remove(context).await,
            SkillsSubcommand::Update => handle_skills_update(context).await,
        }
    }
}
