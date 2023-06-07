use log::debug;

use crate::utils::annotations::read_vault_annotation;

use super::{SecretExtractor, SecretInfo};
use std::path::Path;

pub struct ElixirConfigExtractor;
impl SecretExtractor for ElixirConfigExtractor {
    fn extract(file: &Path) -> Vec<SecretInfo> {
        let src = std::fs::read_to_string(file).unwrap();
        let annotations = read_vault_annotation(&src);

        let mut extracted = vec![];

        if !annotations.is_empty() {
            for annotation in annotations {
                if annotation.key == "wukong.mindvalley.dev/config-secrets-location" {
                    if annotation.source != "vault" {
                        debug!("Invalid source: {}", annotation.source);
                        continue;
                    }
                    if annotation.engine != "secret" {
                        debug!("Invalid engine: {}", annotation.engine);
                        continue;
                    }

                    extracted.push(SecretInfo {
                        provider: "bunker".to_string(),
                        kind: "elixir_config".to_string(),
                        src: annotation.secret_path.clone(),
                        destination_file: annotation.destination_file.clone(),
                        name: annotation.secret_name.clone(),
                        annotated_file: file.to_path_buf(),
                    });
                }
            }
        } else {
            eprintln!("🔍 No annotation found in {}", file.to_string_lossy());
        }

        extracted
    }
}