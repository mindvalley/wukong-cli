use crate::error::ApplicationConfigError;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

/// The application config.
#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationConfig {
    pub name: String,
    pub enable: bool,
    pub workflows: Option<ApplicationWorkflowConfig>,
    pub namespaces: Vec<ApplicationNamespaceConfig>,
    pub addons: Option<ApplicationAddonsConfig>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationNamespaceConfig {
    #[serde(rename = "type")]
    pub namespace_type: String,
    pub build: Option<ApplicationNamespaceBuildConfig>,
    pub delivery: Option<ApplicationNamespaceDeliveryConfig>,
    pub appsignal: Option<ApplicationNamespaceAppsignalConfig>,
    pub honeycomb: Option<ApplicationNamespaceHoneycombConfig>,
    pub cloudsql: Option<ApplicationNamespaceCloudsqlConfig>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationNamespaceBuildConfig {
    pub build_workflow: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationNamespaceDeliveryConfig {
    pub target: String,
    pub base_replica: u32,
    pub rollout_strategy: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationNamespaceAppsignalConfig {
    pub enable: bool,
    pub app_id: String,
    pub environment: String,
    pub default_namespace: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationNamespaceHoneycombConfig {
    pub enable: bool,
    pub dataset: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationWorkflowConfig {
    pub provider: String,
    pub excluded_workflows: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationNamespaceCloudsqlConfig {
    pub enable: bool,
    pub project_id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationAddonElixirLivebookConfig {
    pub enable: bool,
    pub allowed_admins: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationAddonsConfig {
    pub elixir_livebook: Option<ApplicationAddonElixirLivebookConfig>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationConfigs {
    pub application: ApplicationConfig,
    #[serde(skip)]
    config_path: PathBuf,
}

impl FromStr for ApplicationConfigs {
    type Err = ApplicationConfigError;

    fn from_str(application_config: &str) -> Result<Self, Self::Err> {
        toml::from_str::<ApplicationConfigs>(application_config)
            .map_err(ApplicationConfigError::BadTomlData)
    }
}

impl ApplicationConfigs {
    pub fn new() -> Result<Self, ApplicationConfigError> {
        let current_dir = std::env::current_dir().expect("Unable to get current working directory");
        let config_path = std::path::Path::new(&current_dir).join(".wukong.toml");

        Ok(Self {
            application: ApplicationConfig::default(),
            config_path,
        })
    }

    pub fn load() -> Result<Self, ApplicationConfigError> {
        let current_dir = std::env::current_dir().expect("Unable to get current working directory");
        let config_path = std::path::Path::new(&current_dir).join(".wukong.toml");

        let content = std::fs::read_to_string(
            config_path
                .to_str()
                .expect("The config file path is not valid"),
        )
        .map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => ApplicationConfigError::NotFound {
                path: ".wukong.toml",
                source: err,
            },
            io::ErrorKind::PermissionDenied => ApplicationConfigError::PermissionDenied {
                path: ".wukong.toml",
                source: err,
            },
            _ => err.into(),
        })?;

        let mut config: ApplicationConfigs =
            toml::from_str(&content).map_err(ApplicationConfigError::BadTomlData)?;
        config.config_path = config_path;

        Ok(config)
    }

    pub fn save(&self) -> Result<(), ApplicationConfigError> {
        let serialized =
            toml::to_string(self).map_err(ApplicationConfigError::SerializeTomlError)?;

        if let Some(config_file_dir) = self.config_path.parent() {
            create_dir_all(config_file_dir)?;
        }

        let mut file = File::create(self.config_path.clone())?;
        file.write_all(&serialized.into_bytes())?;

        Ok(())
    }

    pub fn to_string(&self) -> Result<String, ApplicationConfigError> {
        let serialized =
            toml::to_string(self).map_err(ApplicationConfigError::SerializeTomlError)?;

        Ok(serialized)
    }
}
