use crate::{error::handle_error, Cli, Config as CLIConfig, CONFIG_FILE};
use clap::{ArgEnum, Args, IntoApp, Subcommand};

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
    CollectTelemetry,
    EnableLog,
    LogDir,
}

impl Config {
    pub fn perform_action(&self) {
        let mut cmd = Cli::command();

        match &self.subcommand {
            ConfigSubcommand::List => {
                let config_file = CONFIG_FILE
                    .as_ref()
                    .expect("Unable to identify user's home directory");

                match CLIConfig::load(config_file) {
                    Ok(config) => {
                        println!("{}", toml::to_string(&config).unwrap());
                    }
                    Err(e) => {
                        handle_error(e);
                        std::process::exit(1);
                    }
                }
            }
            ConfigSubcommand::Set {
                config_name,
                config_value,
            } => {
                let config_file = CONFIG_FILE
                    .as_ref()
                    .expect("Unable to identify user's home directory");

                match CLIConfig::load(&config_file) {
                    Ok(mut config) => match config_name {
                        ConfigName::Application => {
                            config.core.application = config_value.to_string();
                            config.save(&config_file).unwrap();
                            println!("Updated property [core/application].");
                        }
                        ConfigName::CollectTelemetry => {
                            config.core.collect_telemetry = config_value.trim().parse().unwrap();
                            config.save(&config_file).unwrap();
                            println!("Updated property [core/collect_telemetry].");
                        }
                        ConfigName::EnableLog => {
                            config.log.enable = config_value.trim().parse().unwrap();
                            config.save(&config_file).unwrap();
                            println!("Updated property [log/enable].");
                        }
                        ConfigName::LogDir => {
                            config.log.log_dir = config_value.to_string();
                            config.save(&config_file).unwrap();
                            println!("Updated property [log/log_dir].");
                        }
                    },
                    Err(e) => {
                        cmd.error(clap::ErrorKind::Io, e).exit();
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
                        cmd.error(clap::ErrorKind::Io, e).exit();
                    }
                }
            }
        };
    }
}
