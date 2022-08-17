use crate::{
    clap_app::ClapApp,
    config::{Config, CONFIG_FILE},
    error::CliError,
};
use clap::{Command, CommandFactory, Parser, ValueHint};
use clap_complete::{
    generate, generate_to,
    shells::{Bash, Zsh},
    Generator, Shell,
};

pub enum ConfigState {
    InitialisedButUnAuthenticated(Config),
    InitialisedAndAuthenticated(Config),
    Uninitialised,
}

pub struct App {
    pub config: ConfigState,
    pub cli: ClapApp,
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

impl App {
    pub fn new<'a>() -> Result<Self, CliError<'a>> {
        let cli = ClapApp::parse();
        let mut cmd = ClapApp::command();

        cmd.set_bin_name("wukong");

        let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("completions/");
        generate_to::<Bash, _, _>(Bash, &mut cmd, "wukong", &outdir);
        generate_to::<Zsh, _, _>(Zsh, &mut cmd, "wukong", &outdir);

        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let config = match Config::load(config_file) {
            Ok(config) => {
                if config.auth.is_none() {
                    ConfigState::InitialisedButUnAuthenticated(config)
                } else {
                    ConfigState::InitialisedAndAuthenticated(config)
                }
            }
            Err(error) => match error {
                CliError::ConfigError(ref config_error) => match config_error {
                    crate::error::ConfigError::NotFound { .. } => ConfigState::Uninitialised,
                    _ => return Err(error),
                },
                _ => return Err(error),
            },
        };

        Ok(Self {
            config,
            cli: ClapApp::parse(),
        })
    }
}
