use crate::{
    commands::ClapApp,
    error::{CliError, ConfigError},
    Config as CLIConfig, CONFIG_FILE,
};
use clap::{error::ErrorKind, Args, CommandFactory, Subcommand, ValueEnum};

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
    EnableLog,
    LogDir,
    WukongApiUrl,
    OktaClientId,
}

impl Config {
    pub fn handle_command(&self) -> Result<bool, CliError> {
        let mut cmd = ClapApp::command();

        match &self.subcommand {
            ConfigSubcommand::List => {
                let config_file = CONFIG_FILE
                    .as_ref()
                    .expect("Unable to identify user's home directory");

                let config = CLIConfig::load(config_file)?;
                println!(
                    "{}",
                    toml::to_string(&config).map_err(ConfigError::SerializeTomlError)?
                );
            }
            ConfigSubcommand::Set {
                config_name,
                config_value,
            } => {
                let config_file = CONFIG_FILE
                    .as_ref()
                    .expect("Unable to identify user's home directory");

                match CLIConfig::load(config_file) {
                    Ok(mut config) => match config_name {
                        ConfigName::Application => {
                            config.core.application = config_value.trim().to_string();
                            config.save(config_file)?;
                            println!("Updated property [core/application].");
                        }
                        ConfigName::EnableLog => {
                            config.log.enable = config_value
                                .trim()
                                .parse()
                                .expect("The value can't be parsed to bool.");
                            config.save(config_file)?;
                            println!("Updated property [log/enable].");
                        }
                        ConfigName::LogDir => {
                            config.log.log_dir = config_value.trim().to_string();
                            config.save(config_file)?;
                            println!("Updated property [log/log_dir].");
                        }
                        ConfigName::WukongApiUrl => {
                            config.core.wukong_api_url = config_value.trim().to_string();
                            config.save(config_file)?;
                            println!("Updated property [core/wukong_api_url].");
                        }
                        ConfigName::OktaClientId => {
                            config.core.okta_client_id = config_value.trim().to_string();
                            config.save(config_file)?;
                            println!("Updated property [core/okta_client_id].");
                        }
                    },
                    Err(e) => {
                        cmd.error(ErrorKind::Io, e).exit();
                    }
                }
            }
            ConfigSubcommand::Get { config_name } => {
                let config_file = CONFIG_FILE
                    .as_ref()
                    .expect("Unable to identify user's home directory");

                match CLIConfig::load(config_file) {
                    Ok(config) => match config_name {
                        ConfigName::Application => println!("{}", config.core.application),
                        ConfigName::EnableLog => println!("{}", config.log.enable),
                        ConfigName::LogDir => println!("{}", config.log.log_dir),
                        ConfigName::WukongApiUrl => println!("{}", config.core.wukong_api_url),
                        ConfigName::OktaClientId => println!("{}", config.core.okta_client_id),
                    },
                    Err(e) => {
                        cmd.error(ErrorKind::Io, e).exit();
                    }
                }
            }
        };

        Ok(true)
    }
}
