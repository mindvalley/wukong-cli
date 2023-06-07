use toml::Value;

use super::{SecretExtractor, SecretInfo};
use std::path::Path;

pub struct WKTomlConfigExtractor;

impl SecretExtractor for WKTomlConfigExtractor {
    fn extract(file: &Path) -> Vec<SecretInfo> {
        let toml_string = std::fs::read_to_string(file).expect("Failed to read config file");

        // Parse the TOML string
        let parsed_toml: Value = toml::from_str(&toml_string).expect("Failed to parse TOML");

        // Access values from the parsed TOML
        let secrets = parsed_toml.get("secrets").expect("secrets not found");

        let mut extracted = vec![];

        if let Some(secrets_array) = secrets.as_array() {
            for secret in secrets_array {
                if let Some(secret_table) = secret.as_table() {
                    for key in secret_table.keys() {
                        let secret_data = secret_table.get(key).unwrap();

                        let source = secret_data["src"].as_str().unwrap().to_string();
                        let value_part = source.split('#').collect::<Vec<&str>>();
                        if value_part.len() != 2 {
                            continue;
                        }
                        let source = value_part[0].to_string();
                        let secret_name = value_part[1].to_string();

                        let splited_source_and_path = source.split(':').collect::<Vec<&str>>();
                        if splited_source_and_path.len() != 2 {
                            continue;
                        }
                        let path_with_engine = splited_source_and_path[1].to_string();

                        let splited_engine_and_path =
                            path_with_engine.split('/').collect::<Vec<&str>>();
                        let (_engine, path) = splited_engine_and_path.split_at(1);

                        let src = path.join("/");

                        extracted.push(SecretInfo {
                            provider: secret_data["provider"].as_str().unwrap().to_string(),
                            kind: secret_data["kind"].as_str().unwrap().to_string(),
                            src,
                            destination_file: secret_data["dst"].as_str().unwrap().to_string(),
                            name: secret_name,
                            annotated_file: file.to_path_buf(),
                        });
                    }
                }
            }
        }

        extracted
    }
}
