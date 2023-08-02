use crate::{error::ExtractError, utils::annotations::read_vault_annotation};

use super::{SecretExtractor, SecretInfo};
use std::path::Path;

/// Extract secret annotations from elixir config file.
///
/// For example,
/// ```elixir
/// # at /a/b/c/dev.exs
/// # wukong.mindvalley.dev/config-secrets-location: vault:secret/wukong-cli/development#dev.secret.exs
/// import_config("local/dev.secrets.exs")
/// ```
///
/// Extract to
/// ```
/// # use wukong_sdk::secret_extractors::SecretInfo;
/// SecretInfo {
///     key: "vault:secret/wukong-cli/development#dev.secret.exs".to_string(),
///     provider: "bunker".to_string(),
///     kind: "elixir_config".to_string(),
///     src: "wukong-cli/development".to_string(),
///     destination_file: "local/dev/secrets.exs".to_string(),
///     name: "dev.secret.exs".to_string(),
///     annotated_file: "/a/b/c/dev.exs".into()
/// };
/// ```
pub struct ElixirConfigExtractor;
impl SecretExtractor for ElixirConfigExtractor {
    fn extract(file: &Path) -> Result<Vec<SecretInfo>, ExtractError> {
        let src = std::fs::read_to_string(file).unwrap();
        let annotations = read_vault_annotation(&src);

        let mut extracted = Vec::new();

        if !annotations.is_empty() {
            for annotation in annotations {
                if annotation.key == "wukong.mindvalley.dev/config-secrets-location"
                    && annotation.source == "vault"
                    && annotation.engine == "secret"
                {
                    extracted.push(SecretInfo {
                        key: annotation.raw,
                        provider: "bunker".to_string(),
                        kind: "elixir_config".to_string(),
                        src: annotation.secret_path.clone(),
                        destination_file: annotation.destination_file.clone(),
                        name: annotation.secret_name.clone(),
                        annotated_file: file.to_path_buf(),
                    });
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
    fn test_elixir_config_extractor() {
        let dir = assert_fs::TempDir::new().unwrap();
        let dev_config_path = dir.path().join("dev.exs");

        let mut dev_config = File::create(&dev_config_path).unwrap();
        writeln!(
            dev_config,
            r#"# Import development secrets
            # wukong.mindvalley.dev/config-secrets-location: vault:secret/wukong-cli/sandboxes#dev.secrets.exs
            import_config("dev.secrets.exs")

            # wukong.mindvalley.dev/config-secrets-location: vault:secret/wukong-cli/sandboxes#app.secrets.exs
            import_config("app/dev.secrets.exs")"#
        ).unwrap();

        let secret_infos = ElixirConfigExtractor::extract(&dev_config_path).unwrap();

        assert_eq!(secret_infos.len(), 2);

        assert_eq!(
            secret_infos[0].key,
            "vault:secret/wukong-cli/sandboxes#dev.secrets.exs"
        );
        assert_eq!(secret_infos[0].name, "dev.secrets.exs");
        assert_eq!(secret_infos[0].src, "wukong-cli/sandboxes");
        assert_eq!(secret_infos[0].destination_file, "dev.secrets.exs");
        assert_eq!(secret_infos[0].provider, "bunker");
        assert_eq!(secret_infos[0].kind, "elixir_config");
        assert_eq!(secret_infos[0].annotated_file, dev_config_path);

        assert_eq!(
            secret_infos[1].key,
            "vault:secret/wukong-cli/sandboxes#app.secrets.exs"
        );
        assert_eq!(secret_infos[1].name, "app.secrets.exs");
        assert_eq!(secret_infos[1].src, "wukong-cli/sandboxes");
        assert_eq!(secret_infos[1].destination_file, "app/dev.secrets.exs");
        assert_eq!(secret_infos[1].provider, "bunker");
        assert_eq!(secret_infos[1].kind, "elixir_config");
        assert_eq!(secret_infos[1].annotated_file, dev_config_path);

        dir.close().unwrap();
    }
}
