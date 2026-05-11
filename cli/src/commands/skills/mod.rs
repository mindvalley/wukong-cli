mod init;
mod publish;
mod remove;

use clap::{Args, Subcommand};

use crate::error::WKCliError;

use self::{
    init::handle_skills_init, publish::handle_skills_publish, remove::handle_skills_remove,
};

use super::Context;

#[derive(Debug, Args)]
pub struct Skills {
    #[command(subcommand)]
    pub subcommand: SkillsSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum SkillsSubcommand {
    /// Scaffold a new skill at ./.claude/skills/<name>/SKILL.md (interactive)
    Init,
    /// Publish a local skill to the internal skills registry repo
    Publish,
    /// Remove installed skills from local and global skill directories
    Remove,
}

impl Skills {
    pub async fn handle_command(&self, context: Context) -> Result<bool, WKCliError> {
        match &self.subcommand {
            SkillsSubcommand::Init => handle_skills_init(context).await,
            SkillsSubcommand::Publish => handle_skills_publish(context).await,
            SkillsSubcommand::Remove => handle_skills_remove(context).await,
        }
    }
}
