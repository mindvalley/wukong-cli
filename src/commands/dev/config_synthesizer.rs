use crate::{error::CliError, services::vault::Vault, utils::annotations::read_vault_annotation};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::io::prelude::*;
use std::{env::current_dir, fs::File, path::PathBuf};

pub async fn handle_config_synthesizer(path: &PathBuf) -> Result<bool, CliError> {
    let lint_path = match path.try_exists() {
        Ok(true) => {
            if path.to_string_lossy() == "." {
                current_dir().unwrap()
            } else {
                path.to_path_buf()
            }
        }
        Ok(false) => {
            eprintln!("path '{}' does not exist", path.to_string_lossy());
            panic!();
        }
        Err(_) => todo!(),
    };

    let mut overrides = OverrideBuilder::new(&lint_path);
    overrides.add("**/config/dev.exs").unwrap();

    let available_files: Vec<PathBuf> = WalkBuilder::new(&lint_path)
        .overrides(overrides.build().unwrap())
        .build()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    println!("available_files: {:#?}", available_files);

    for file in available_files {
        let src = std::fs::read_to_string(file.clone()).unwrap();
        let annotations = read_vault_annotation(&src);

        if !annotations.is_empty() {
            // println!("Found annotations in file: {}", file.to_string_lossy());
            for annotation in annotations {
                let vault = Vault::new();
                let vault_token = vault.get_token_or_login().await.unwrap();
                println!("vault_token: {:#?}", vault_token);

                println!("annotation: {:#?}", annotation);

                if annotation.0 .0 == "wukong.mindvalley.dev/config-secrets-location" {
                    let path = annotation.0 .1.clone();
                    let local_secret_path = annotation.1.to_string();

                    let path_part = path.split("#").collect::<Vec<&str>>();
                    let path = path_part[0].to_string();
                    let key = path_part[1].to_string();
                    // TODO: check the first part is "vault"
                    let vault_path_part = path.split(":").collect::<Vec<&str>>();
                    let vault_secret_path = vault_path_part[1].to_string();
                    println!("path: {:#?}", vault_secret_path);
                    println!("local_secret_path: {:#?}", local_secret_path);

                    let secret = vault
                        .get_secret(&vault_token, &vault_secret_path, &key)
                        .await;
                    println!("secrets: {:#?}", secret);

                    let file_path = file.parent().unwrap().join(local_secret_path.clone());
                    if local_secret_path.contains("/") {
                        let dir_path = file_path.parent().unwrap();
                        std::fs::create_dir_all(dir_path).unwrap();
                    }
                    println!("file_path: {:#?}", file_path);
                    let mut file = File::create(file_path).unwrap();
                    file.write_all(secret.unwrap().as_bytes()).unwrap();
                }
            }
        }
    }

    Ok(true)
}
