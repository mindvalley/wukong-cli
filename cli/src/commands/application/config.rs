use crate::error::ApplicationConfigError;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
};

/// The application config.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationConfig {
    pub name: String,
    pub enable: bool,
    pub workflows: Option<ApplicationWorkflowConfig>,
    pub namespaces: Vec<ApplicationNamespaceConfig>,
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
    pub enable: bool,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationNamespaceCloudsqlConfig {
    pub enable: bool,
    pub project_id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationConfigs {
    pub application: Option<ApplicationConfig>,
    #[serde(skip)]
    config_path: PathBuf,
}

impl ApplicationConfigs {
    pub fn new() -> Result<Self, ApplicationConfigError> {
        let current_dir = std::env::current_dir().expect("Unable to get current working directory");
        let config_path = std::path::Path::new(&current_dir).join(".wukong.toml");

        if let Ok(file) = std::fs::read_to_string(
            config_path
                .to_str()
                .expect("The config file path is not valid"),
        ) {
            let mut config: ApplicationConfigs =
                toml::from_str(&file).map_err(ApplicationConfigError::BadTomlData)?;

            config.config_path = config_path;

            return Ok(config);
        }

        Ok(Self {
            application: None,
            config_path,
        })
    }

    // pub fn sync_

    pub fn save(&self) -> Result<(), ApplicationConfigError> {
        println!("Saving config to {:?}", self.config_path);
        let config_dir = self
            .config_path
            .parent()
            .expect("Failed to get parent directory");

        // Ensure directory exists
        create_dir_all(config_dir)?;

        // Use temporary file to achieve atomic write:
        //  1. Open file /.wukong.toml
        //  2. Serialize config to temporary file
        //  3. Rename temporary file to /.config.toml (atomic operation)
        let tmp_file_path = self.config_path.with_extension("tmp");
        let mut tmp_file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_file_path)?;

        let serialized =
            toml::to_string(self).map_err(ApplicationConfigError::SerializeTomlError)?;

        tmp_file.write_all(&serialized.into_bytes())?;

        // Rename file to final destination to achieve atomic write
        std::fs::rename(tmp_file_path.as_path(), &self.config_path).expect("Failed to rename file");
        // let toml = toml::to_string(&self).map_err(ApplicationConfigError::BadTomlData)?;
        // std::fs::write(&self.config_path, toml).map_err(ApplicationConfigError::IoError);

        Ok(())
    }

    // /// Load a configuration from file.
    // ///
    // /// # Errors
    // ///
    // /// This function may return typical file I/O errors.
    // pub fn load_from_path(path: &'static str) -> Result<Self, ConfigError> {
    //     let config_file_path = Path::new(path);

    //     let content = std::fs::read_to_string(
    //         config_file_path
    //             .to_str()
    //             .expect("The config file path is not valid."),
    //     )
    //     .map_err(|err| match err.kind() {
    //         io::ErrorKind::NotFound => ConfigError::NotFound { path, source: err },
    //         io::ErrorKind::PermissionDenied => ConfigError::PermissionDenied { path, source: err },
    //         _ => err.into(),
    //     })?;

    //     let config = toml::from_str(&content).map_err(ConfigError::BadTomlData)?;

    //     Ok(config)
    // }

    // /// Save a configuration to file.
    // ///
    // /// If the file's directory does not exist, it will be created. If the file
    // /// already exists, it will be overwritten.
    // ///
    // /// # Errors
    // ///
    // /// This function may return typical file I/O errors.
    // pub fn save_to_path(&self, path: &str) -> Result<(), ConfigError> {
    //     let config_file_path = Path::new(path);
    //     let serialized = toml::to_string(self).map_err(ConfigError::SerializeTomlError)?;

    //     if let Some(outdir) = config_file_path.parent() {
    //         create_dir_all(outdir)?;
    //     }
    //     let mut file = File::create(path)?;
    //     file.write_all(&serialized.into_bytes())?;

    //     Ok(())
    // }

    // pub fn save_to_default_path(&self) -> Result<(), ConfigError> {
    //     self.save_to_path(
    //         CONFIG_FILE
    //             .as_ref()
    //             .expect("Unable to identify user's home directory"),
    //     )
    // }

    // pub fn get_current_directory(&self) -> Result<String, ConfigError> {
    //     let current_dir = std::env::current_dir()?;
    //     let path = current_dir
    //         .to_str()
    //         .expect("Unable to get current working directory");

    //     Ok(path.to_owned())
    // }
}
