use crate::{Config as CLIConfig, CONFIG_FILE};
use clap::{ArgEnum, Args, Subcommand};

#[derive(Debug, Args)]
pub struct Config {
    #[clap(subcommand)]
    pub subcommand: ConfigSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// List the configurations
    List,
    /// Set the value of a configuration
    Set {
        /// The config name
        #[clap(arg_enum)]
        config_name: ConfigName,
        /// The config value
        config_value: String,
    },
    /// Print the value of a configuration
    Get {
        /// The config name
        #[clap(arg_enum)]
        config_name: ConfigName,
    },
}

#[derive(Debug, ArgEnum, Clone)]
pub enum ConfigName {
    Application,
    EnableTelemetry,
    EnableLog,
    LogDir,
}

impl Config {
    pub fn perform_action(&self) {
        match &self.subcommand {
            ConfigSubcommand::List => {
                if let Some(ref config_file) = *CONFIG_FILE {
                    let config = CLIConfig::load(config_file).unwrap();
                    println!("{}", toml::to_string(&config).unwrap());
                }
            }
            ConfigSubcommand::Set {
                config_name,
                config_value,
            } => {
                if let Some(ref config_file) = *CONFIG_FILE {
                    let mut config = CLIConfig::load(config_file).unwrap();

                    match config_name {
                        ConfigName::Application => {
                            println!("{}", config.core.application);
                            config.core.application = config_value.to_string();
                            config.save(config_file).unwrap();
                        }
                        ConfigName::EnableTelemetry => {
                            println!("{}", config.core.collect_telemetry);
                            // config.core.application = config_value.to_string();
                            // config.save(config_file).unwrap();
                        }
                        ConfigName::EnableLog => {
                            println!("{}", config.log.enable);
                            // config.core.application = config_value.to_string();
                            // config.save(config_file).unwrap();
                        }
                        ConfigName::LogDir => {
                            println!("{}", config.log.log_dir);
                            config.log.log_dir = config_value.to_string();
                            config.save(config_file).unwrap();
                        }
                    }
                }
            }
            ConfigSubcommand::Get { config_name } => {
                if let Some(ref config_file) = *CONFIG_FILE {
                    let config = CLIConfig::load(config_file).unwrap();

                    match config_name {
                        ConfigName::Application => println!("{}", config.core.application),
                        ConfigName::EnableTelemetry => {
                            println!("{}", config.core.collect_telemetry);
                        }
                        ConfigName::EnableLog => println!("{}", config.log.enable),
                        ConfigName::LogDir => println!("{}", config.log.log_dir),
                    }
                }
            }
        };
    }
}
