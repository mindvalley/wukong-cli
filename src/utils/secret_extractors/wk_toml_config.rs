use super::{SecretExtractor, SecretInfo};
use crate::error::ExtractError;
use owo_colors::OwoColorize;
use std::path::Path;
use toml::Value;

// at /a/b/c/.wukong.toml
// [secrets.dotenv]
// provider = "bunker"
// kind = "generic"
// src = "vault:secret/wukong-cli/development#dotenv"
// dst = ".env"
//
// Extract to
// SecretInfo {
//     key: "dotenv",
//     provider: "bunker",
//     kind: "generic",
//     src: "wukong-cli/development",
//     dst: ".env",
//     name: "dotenv",
//     annotated_file: /a/b/c/.wukong.toml
// }
pub struct WKTomlConfigExtractor;

impl SecretExtractor for WKTomlConfigExtractor {
    fn extract(file: &Path) -> Result<Vec<SecretInfo>, ExtractError> {
        let toml_string = std::fs::read_to_string(file).expect("Failed to read toml file");

        // Parse the TOML string
        let parsed_toml: Value = toml::from_str(&toml_string)?;

        // Access values from the parsed TOML
        let mut extracted = Vec::new();
        if let Some(secrets) = parsed_toml.get("secrets") {
            if let Some(secrets_array) = secrets.as_array() {
                for secret in secrets_array {
                    if let Some(secret_table) = secret.as_table() {
                        for key in secret_table.keys() {
                            let secret_data = secret_table.get(key).unwrap();

                            let provider = secret_data["provider"].as_str().unwrap().to_string();
                            let kind = secret_data["kind"].as_str().unwrap().to_string();
                            let source = secret_data["src"].as_str().unwrap().to_string();

                            // we are ignoring configs if provider is not "bunker" and kind is not
                            // "generic" for now
                            if provider != "bunker" || kind != "generic" {
                                continue;
                            }

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
                            let (engine, path) = splited_engine_and_path.split_at(1);

                            if (splited_source_and_path[0] != "vault") || (engine[0] != "secret") {
                                continue;
                            }

                            let destination_file = secret_data["dst"].as_str().unwrap().to_string();
                            if (destination_file.starts_with("~/"))
                                || (destination_file.starts_with("/"))
                            {
                                eprintln!(
                                "⚠️  The dst {} is not under the project directory. It will be ignored.",
                                destination_file.cyan()
                            );
                                continue;
                            }

                            let src = path.join("/");

                            extracted.push(SecretInfo {
                                key: key.to_string(),
                                provider,
                                kind,
                                src,
                                destination_file,
                                name: secret_name,
                                annotated_file: file.to_path_buf(),
                            });
                        }
                    }
                }
            }
        }

        Ok(extracted)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_wk_toml_config_extractor() {
        let dir = assert_fs::TempDir::new().unwrap();
        let dev_config_path = dir.path().join("dev.exs");

        let mut dev_config = File::create(&dev_config_path).unwrap();
        writeln!(
            dev_config,
            r#"
[[secrets]]

[secrets.dotenv]
provider = "bunker"
kind = "generic"
src = "vault:secret/wukong-cli/development#dotenv"
dst = ".env"

[secrets.kubeconfig]
provider = "bunker"
kind = "generic"
src = "vault:secret/wukong-cli/development#kubeconfig"
dst = "priv/files/kubeconfig"
            "#
        )
        .unwrap();

        let secret_infos = WKTomlConfigExtractor::extract(&dev_config_path).unwrap();

        assert_eq!(secret_infos.len(), 2);

        assert_eq!(secret_infos[0].key, "dotenv");
        assert_eq!(secret_infos[0].name, "dotenv");
        assert_eq!(secret_infos[0].src, "wukong-cli/development");
        assert_eq!(secret_infos[0].destination_file, ".env");
        assert_eq!(secret_infos[0].provider, "bunker");
        assert_eq!(secret_infos[0].kind, "generic");
        assert_eq!(secret_infos[0].annotated_file, dev_config_path);

        assert_eq!(secret_infos[1].key, "kubeconfig");
        assert_eq!(secret_infos[1].name, "kubeconfig");
        assert_eq!(secret_infos[1].src, "wukong-cli/development");
        assert_eq!(secret_infos[1].destination_file, "priv/files/kubeconfig");
        assert_eq!(secret_infos[1].provider, "bunker");
        assert_eq!(secret_infos[1].kind, "generic");
        assert_eq!(secret_infos[1].annotated_file, dev_config_path);

        dir.close().unwrap();
    }
}
