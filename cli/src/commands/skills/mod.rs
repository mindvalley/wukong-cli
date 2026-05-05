mod init;
mod remove;

use clap::{Args, Subcommand};

use crate::error::WKCliError;

use self::{init::handle_skills_init, remove::handle_skills_remove};

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
    /// Remove installed skills from local and global skill directories
    Remove,
}

impl Skills {
    pub async fn handle_command(&self, context: Context) -> Result<bool, WKCliError> {
        match &self.subcommand {
            SkillsSubcommand::Init => handle_skills_init(context).await,
            SkillsSubcommand::Remove => handle_skills_remove(context).await,
        }
    }
}
