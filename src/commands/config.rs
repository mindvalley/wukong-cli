use crate::{
    clap_app::ClapApp,
    error::{CliError, ConfigError},
    Config as CLIConfig, GlobalContext, CONFIG_FILE,
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
    CollectTelemetry,
    EnableLog,
    LogDir,
}

impl Config {
    pub fn handle_command(&self, _context: GlobalContext) -> Result<bool, CliError> {
        let mut cmd = ClapApp::command();

        match &self.subcommand {
            ConfigSubcommand::List => {
                let config_file = CONFIG_FILE
                    .as_ref()
                    .expect("Unable to identify user's home directory");

                let config = CLIConfig::load(config_file)?;
                println!(
                    "{}",
                    toml::to_string(&config).map_err(|err| ConfigError::SerializeTomlError(err))?
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
                            config.core.application = config_value.to_string();
                            config.save(config_file)?;
                            println!("Updated property [core/application].");
                        }
                        ConfigName::CollectTelemetry => {
                            config.core.collect_telemetry = config_value.trim().parse().unwrap();
                            config.save(config_file)?;
                            println!("Updated property [core/collect_telemetry].");
                        }
                        ConfigName::EnableLog => {
                            config.log.enable = config_value.trim().parse().unwrap();
                            config.save(config_file)?;
                            println!("Updated property [log/enable].");
                        }
                        ConfigName::LogDir => {
                            config.log.log_dir = config_value.to_string();
                            config.save(config_file)?;
                            println!("Updated property [log/log_dir].");
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
                        ConfigName::CollectTelemetry => {
                            println!("{}", config.core.collect_telemetry);
                        }
                        ConfigName::EnableLog => println!("{}", config.log.enable),
                        ConfigName::LogDir => println!("{}", config.log.log_dir),
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
