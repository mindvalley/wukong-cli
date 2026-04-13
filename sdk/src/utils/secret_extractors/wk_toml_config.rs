use super::{SecretExtractor, SecretInfo};
use crate::error::ExtractError;
use owo_colors::OwoColorize;
use std::path::Path;
use toml::Value;

/// Extract secret configs from `.wukong.toml` file.
///
/// For example,
/// ```toml
/// # at /a/b/c/.wukong.toml
/// [secrets.dotenv]
/// provider = "bunker"
/// kind = "generic"
/// src = "vault:secret/wukong-cli/development#dotenv"
/// dst = ".env"
/// ```
///
/// Extract to
/// ```
/// # use wukong_sdk::secret_extractors::SecretInfo;
/// SecretInfo {
///     key: "dotenv".to_string(),
///     provider: "bunker".to_string(),
///     kind: "generic".to_string(),
///     src: "wukong-cli/development".to_string(),
///     destination_file: ".env".to_string(),
///     name: "dotenv".to_string(),
///     annotated_file: "/a/b/c/.wukong.toml".into()
/// };
/// ```
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
                        for (key, value) in secret_table.iter() {
                            let provider = match value.get("provider") {
                                Some(v) => v.as_str().unwrap().to_string(),
                                None => {
                                    eprintln!(
                                        "⚠️  [wukong_toml] The {} not found under {} table. It will be ignored.",
                                        "provider".cyan(),
                                        key.cyan()
                                    );
                                    continue;
                                }
                            };
                            let kind = match value.get("kind") {
                                Some(v) => v.as_str().unwrap().to_string(),
                                None => {
                                    eprintln!(
                                        "⚠️  [wukong_toml] The {} not found under {} table. It will be ignored.",
                                        "kind".cyan(),
                                        key.cyan()
                                    );
                                    continue;
                                }
                            };
                            let source = match value.get("src") {
                                Some(v) => v.as_str().unwrap().to_string(),
                                None => {
                                    eprintln!(
                                        "⚠️  [wukong_toml] The {} not found under {} table. It will be ignored.",
                                        "src".cyan(),
                                        key.cyan()
                                    );
                                    continue;
                                }
                            };
                            let destination_file = match value.get("dst") {
                                Some(v) => v.as_str().unwrap().to_string(),
                                None => {
                                    eprintln!(
                                        "⚠️  [wukong_toml] The {} not found under {} table. It will be ignored.",
                                        "dst".cyan(),
                                        key.cyan()
                                    );
                                    continue;
                                }
                            };

                            if kind != "generic" {
                                continue;
                            }

                            // Split the `#fragment` off `src` to get the secret name.
                            // This is shared between bunker (`vault:secret/PATH#NAME`)
                            // and url (`https://HOST/PATH#NAME`) providers.
                            let value_part = source.split('#').collect::<Vec<&str>>();
                            if value_part.len() != 2 {
                                continue;
                            }
                            let source = value_part[0].to_string();
                            let secret_name = value_part[1].to_string();

                            let src = match provider.as_str() {
                                "bunker" => {
                                    let splited_source_and_path =
                                        source.split(':').collect::<Vec<&str>>();
                                    if splited_source_and_path.len() != 2 {
                                        continue;
                                    }
                                    let path_with_engine =
                                        splited_source_and_path[1].to_string();

                                    let splited_engine_and_path =
                                        path_with_engine.split('/').collect::<Vec<&str>>();
                                    let (engine, path) = splited_engine_and_path.split_at(1);

                                    if (splited_source_and_path[0] != "vault")
                                        || (engine[0] != "secret")
                                    {
                                        continue;
                                    }

                                    path.join("/")
                                }
                                "wukong" => {
                                    // Expect: wukong:APPLICATION/NAMESPACE/PATH...#KEY
                                    // (`#KEY` was already split into `secret_name`)
                                    let scheme_split =
                                        source.split(':').collect::<Vec<&str>>();
                                    if scheme_split.len() != 2 || scheme_split[0] != "wukong" {
                                        eprintln!(
                                            "⚠️  [wukong_toml] The {} entry has provider = \"wukong\" but {} does not match {}. It will be ignored.",
                                            key.cyan(),
                                            "src".cyan(),
                                            "wukong:APPLICATION/NAMESPACE/PATH#KEY".cyan()
                                        );
                                        continue;
                                    }
                                    let segments =
                                        scheme_split[1].split('/').collect::<Vec<&str>>();
                                    if segments.len() < 3
                                        || segments.iter().any(|s| s.is_empty())
                                    {
                                        eprintln!(
                                            "⚠️  [wukong_toml] The {} entry needs at least application/namespace/path after the {} scheme. It will be ignored.",
                                            key.cyan(),
                                            "wukong:".cyan()
                                        );
                                        continue;
                                    }
                                    // Store the slash-joined APP/NS/PATH form in `src`;
                                    // the dispatch sites split it back into the
                                    // (application, namespace, path) tuple.
                                    scheme_split[1].to_string()
                                }
                                _ => {
                                    eprintln!(
                                        "⚠️  [wukong_toml] Unknown provider {:?} under {} table. It will be ignored.",
                                        provider,
                                        key.cyan()
                                    );
                                    continue;
                                }
                            };

                            if (destination_file.starts_with("~/"))
                                || (destination_file.starts_with('/'))
                            {
                                eprintln!(
                                "⚠️  [wukong_toml] The destination {} is not under the project directory. It will be ignored.",
                                destination_file.cyan()
                                );
                                continue;
                            }

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

    #[test]
    fn test_wk_toml_config_extractor_wukong_provider() {
        let dir = assert_fs::TempDir::new().unwrap();
        let dev_config_path = dir.path().join(".wukong.toml");

        let mut dev_config = File::create(&dev_config_path).unwrap();
        writeln!(
            dev_config,
            r#"
[[secrets]]

[secrets.dotenv]
provider = "wukong"
kind = "generic"
src = "wukong:mv-platform/staging/wukong-cli/development#dotenv"
dst = ".env"

[secrets.too_short]
provider = "wukong"
kind = "generic"
src = "wukong:onlyone#foo"
dst = "foo.txt"

[secrets.bad_scheme]
provider = "wukong"
kind = "generic"
src = "vault:secret/wukong-cli/development#bar"
dst = "bar.txt"
            "#
        )
        .unwrap();

        let secret_infos = WKTomlConfigExtractor::extract(&dev_config_path).unwrap();

        assert_eq!(secret_infos.len(), 1);
        assert_eq!(secret_infos[0].key, "dotenv");
        assert_eq!(secret_infos[0].name, "dotenv");
        assert_eq!(
            secret_infos[0].src,
            "mv-platform/staging/wukong-cli/development"
        );
        assert_eq!(secret_infos[0].destination_file, ".env");
        assert_eq!(secret_infos[0].provider, "wukong");
        assert_eq!(secret_infos[0].kind, "generic");

        dir.close().unwrap();
    }
}
