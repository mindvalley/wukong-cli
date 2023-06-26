use crate::{
    error::{CliError, ConfigError},
    Config as CLIConfig,
};
use clap::{Args, Subcommand, ValueEnum};

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
    Application,
    WukongApiUrl,
    OktaClientId,
}

impl Config {
    pub fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            ConfigSubcommand::List => {
                let config = CLIConfig::load_from_default_path()?;

                println!(
                    "{}",
                    toml::to_string(&config).map_err(ConfigError::SerializeTomlError)?
                );
            }
            ConfigSubcommand::Set {
                config_name,
                config_value,
            } => {
                let mut config = CLIConfig::load_from_default_path()?;
                match config_name {
                    ConfigName::Application => {
                        config.core.application = config_value.trim().to_string();
                        config.save_to_default_path()?;
                        println!("Updated property [core/application].");
                    }
                    ConfigName::WukongApiUrl => {
                        config.core.wukong_api_url = config_value.trim().to_string();
                        config.save_to_default_path()?;
                        println!("Updated property [core/wukong_api_url].");
                    }
                    ConfigName::OktaClientId => {
                        config.core.okta_client_id = config_value.trim().to_string();
                        config.save_to_default_path()?;
                        println!("Updated property [core/okta_client_id].");
                    }
                };
            }
            ConfigSubcommand::Get { config_name } => {
                let config = CLIConfig::load_from_default_path()?;
                match config_name {
                    ConfigName::Application => println!("{}", config.core.application),
                    ConfigName::WukongApiUrl => println!("{}", config.core.wukong_api_url),
                    ConfigName::OktaClientId => println!("{}", config.core.okta_client_id),
                };
            }
        };

        Ok(true)
    }
}
