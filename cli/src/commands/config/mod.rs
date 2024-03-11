use crate::{commands::config::list::handle_list, error::WKCliError};
use clap::{Args, Subcommand, ValueEnum};

use self::{get::handle_get, set::handle_set};

mod get;
mod list;
mod set;

#[derive(Debug, Args)]
pub struct Config {
    #[command(subcommand)]
    pub subcommand: ConfigSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// List the configurations
    List,
    /// Set the value of a configuration
    Set {
        /// The config name
        #[arg(value_enum)]
        config_name: ConfigName,
        /// The config value
        config_value: String,
    },
    /// Print the value of a configuration
    Get {
        /// The config name
        #[arg(value_enum)]
        config_name: ConfigName,
    },
}

#[derive(Debug, ValueEnum, Clone)]
pub enum ConfigName {
    WukongApiUrl,
    OktaClientId,
}

impl Config {
    pub fn handle_command(&self) -> Result<bool, WKCliError> {
        match &self.subcommand {
            ConfigSubcommand::List => handle_list(),
            ConfigSubcommand::Set {
                config_name,
                config_value,
            } => handle_set(config_name, config_value),
            ConfigSubcommand::Get { config_name } => handle_get(config_name),
        }
    }
}
