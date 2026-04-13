use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    fs::{File, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use log::debug;
use owo_colors::OwoColorize;
use wukong_sdk::services::vault::client::FetchSecretsData;

use crate::{
    commands::{dev::config::utils::get_local_config_path, Context},
    config::Config,
    error::WKCliError,
    wukong_client::WKClient,
};

use super::diff::has_diff;
use super::utils::{
    extract_secret_infos, get_secret_config_files, parse_wukong_src, vault_token_for,
};
use wukong_telemetry::*;
use wukong_telemetry_macro::*;

/// Suffix appended to the original filename when a pull would overwrite it.
const BACKUP_SUFFIX: &str = ".bak";

/// Pattern we ensure is present in `.gitignore` so backups never get
/// accidentally committed.
const BACKUP_GITIGNORE_PATTERN: &str = "*.bak";

#[wukong_telemetry(command_event = "dev_config_pull")]
pub async fn handle_config_pull(context: Context, path: &Path) -> Result<bool, WKCliError> {
    let path = path.try_exists().map(|value| match value {
        true => {
            if path.to_string_lossy() == "." {
                current_dir()
            } else {
                Ok(path.to_path_buf())
            }
        }
        false => Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("path '{}' does not exist", path.to_string_lossy()),
        )),
    })??;

    let secret_config_files = get_secret_config_files(Some(path))?;
    let extracted_infos = extract_secret_infos(secret_config_files)?;

    let mut config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;
    let vault_token = vault_token_for(&extracted_infos, &mut config).await?;

    let mut has_error = false;
    let mut secrets_cache: HashMap<(String, String), FetchSecretsData> = HashMap::new();
    // Tracks which `.gitignore` files we've already touched this run so we
    // don't re-read/append them per annotation.
    let mut gitignored_dirs: HashSet<PathBuf> = HashSet::new();

    for info in extracted_infos {
        eprintln!();
        eprintln!("🔍 {} annotation(s) found in {}", info.1.len(), info.0);

        for annotation in info.1 {
            let source_path = annotation.src.clone();
            let destination_path = annotation.destination_file.clone();
            let cache_key = (annotation.provider.clone(), source_path.clone());

            let file_path = get_local_config_path(&destination_path, &info.0);

            // cache the secrets so we don't call the remote multiple times for the
            // same (provider, path)
            let secret = match secrets_cache.get(&cache_key) {
                Some(secrets) => match secrets.data.get(&annotation.name) {
                    Some(secret) => secret.to_string(),
                    None => {
                        debug!("Secret not found: {:?}", annotation.name);
                        eprintln!(
                            "\t{} {} {} {}",
                            "Not created".red(),
                            file_path.to_string_lossy(),
                            "because".bold(),
                            "Secret not found".bold().red()
                        );
                        has_error = true;
                        continue;
                    }
                },
                None => {
                    let fetch_result = if annotation.provider == "wukong" {
                        let (app, ns, path) = parse_wukong_src(&source_path);
                        wk_client.get_wukong_secrets(&app, &ns, &path).await
                    } else {
                        wk_client.get_secrets(&vault_token, &source_path).await
                    };
                    let secrets = match fetch_result {
                        Ok(secrets) => secrets,
                        Err(err) => {
                            debug!("Error while fetching secrets: {:?}", &source_path);
                            eprintln!(
                                "\t{} {} {} {}",
                                "Not created".red(),
                                file_path.to_string_lossy(),
                                "because".bold(),
                                err.bold().red()
                            );
                            has_error = true;
                            continue;
                        }
                    };
                    secrets_cache.insert(cache_key.clone(), secrets);

                    match secrets_cache
                        .get(&cache_key)
                        .unwrap()
                        .data
                        .get(&annotation.name)
                    {
                        Some(secret) => secret.to_string(),
                        None => {
                            debug!("Secret not found: {:?}", annotation.name);
                            eprintln!(
                                "\t{} {} {} {}",
                                "Not created".red(),
                                file_path.to_string_lossy(),
                                "because".bold(),
                                "Secret not found".bold().red()
                            );
                            has_error = true;
                            continue;
                        }
                    }
                }
            };

            if destination_path.contains('/') {
                let dir_path = file_path.parent().unwrap();
                if let Err(err) = std::fs::create_dir_all(dir_path) {
                    debug!("Error while creating directory: {:?}", err);
                    eprintln!(
                        "\t{} {} {} {}",
                        "Not created".red(),
                        file_path.to_string_lossy(),
                        "because".bold(),
                        err.to_string().bold().red()
                    );
                    has_error = true;
                    continue;
                };
            }

            // Snapshot the previous version before we overwrite it, but only
            // when the file actually exists and its content differs from what
            // we're about to write. First-pull (file absent) and no-op pull
            // (identical content) both skip backup.
            if let Some(backup_path) = backup_existing_if_changed(&file_path, &secret) {
                eprintln!(
                    "\t{} {}",
                    "📦 Backed up previous version to".cyan(),
                    backup_path.to_string_lossy()
                );
                ensure_backup_gitignored(&file_path, &info.0, &mut gitignored_dirs);
            }

            match File::create(&file_path) {
                Ok(mut file) => {
                    if let Err(err) = file.write_all(secret.as_bytes()) {
                        debug!("Error while creating file: {:?}", err);
                        eprintln!(
                            "\t{} {} {} {}",
                            "Not created".red(),
                            file_path.to_string_lossy(),
                            "because".bold(),
                            err.to_string().bold().red()
                        );
                        has_error = true;
                        continue;
                    };
                }
                Err(err) => {
                    debug!("Error while writing file: {:?}", err);
                    eprintln!(
                        "\t{} {} {} {}",
                        "Not created".red(),
                        file_path.to_string_lossy(),
                        "because".bold(),
                        err.to_string().bold().red()
                    );
                    has_error = true;
                    continue;
                }
            }

            eprintln!("\t{} {}", "Created".green(), file_path.to_string_lossy());
        }
    }

    if has_error {
        Ok(false)
    } else {
        Ok(true)
    }
}

/// If `file_path` already exists and its content differs from `incoming`,
/// copy it to a sibling `<name>.bak` (overwriting any prior backup) and
/// return that path. Otherwise return `None` and write nothing.
///
/// Errors during read/write are logged and treated as "no backup" — we never
/// abort the pull just because a backup attempt failed.
fn backup_existing_if_changed(file_path: &Path, incoming: &str) -> Option<PathBuf> {
    let existing = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        // Includes the file-not-found case (first pull) and any other read
        // failure. We don't try to distinguish — both mean "no backup needed
        // or possible".
        Err(err) => {
            debug!(
                "skipping backup of {}: {:?}",
                file_path.to_string_lossy(),
                err
            );
            return None;
        }
    };

    if !has_diff(&existing, incoming) {
        return None;
    }

    // Build the backup path via OsString so non-UTF-8 filenames survive.
    let mut name = file_path.file_name()?.to_os_string();
    name.push(BACKUP_SUFFIX);
    let backup = file_path.with_file_name(name);

    if let Err(err) = std::fs::copy(file_path, &backup) {
        debug!(
            "failed to write backup {}: {:?}",
            backup.to_string_lossy(),
            err
        );
        return None;
    }

    Some(backup)
}

/// Append `*.bak` to the nearest `.gitignore` (walking up from the
/// destination file, stopping at the directory containing the annotated
/// config). If no `.gitignore` exists along that chain, create one next to
/// the annotated config. Idempotent across the run via `seen_dirs`, and
/// idempotent across runs via a per-line check.
fn ensure_backup_gitignored(
    file_path: &Path,
    annotated_config_path: &str,
    seen_dirs: &mut HashSet<PathBuf>,
) {
    let config_dir = match Path::new(annotated_config_path).parent() {
        Some(p) => p.to_path_buf(),
        None => return,
    };

    let target = find_gitignore_target(file_path, &config_dir);

    if !seen_dirs.insert(target.clone()) {
        return;
    }

    // Single read covers both the presence check and the trailing-newline
    // decision below. Missing file → empty string, which means "create".
    let content = std::fs::read_to_string(&target).unwrap_or_default();
    if content
        .lines()
        .any(|line| line.trim() == BACKUP_GITIGNORE_PATTERN)
    {
        return;
    }

    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&target)
        .and_then(|mut f| {
            // Avoid running the new pattern onto the previous line when the
            // existing file lacks a trailing newline. An empty file gets no
            // leading newline.
            let needs_separator = !content.is_empty() && !content.ends_with('\n');
            let prefix = if needs_separator { "\n" } else { "" };
            writeln!(f, "{}{}", prefix, BACKUP_GITIGNORE_PATTERN)
        });

    match result {
        Ok(_) => eprintln!(
            "\t{} {} {}",
            "🔒 Added".cyan(),
            BACKUP_GITIGNORE_PATTERN.bold(),
            format!("to {}", target.to_string_lossy()).cyan()
        ),
        Err(err) => debug!("failed to update {}: {:?}", target.to_string_lossy(), err),
    }
}

/// Walk upward from `file_path`'s directory until we find an existing
/// `.gitignore`, stopping at `config_dir`. If none is found (or if the file
/// somehow lives outside `config_dir`), fall back to creating one in
/// `config_dir` itself — we never modify a `.gitignore` above the project's
/// annotated-config root.
fn find_gitignore_target(file_path: &Path, config_dir: &Path) -> PathBuf {
    let start = file_path.parent().unwrap_or(config_dir);

    // Defensive: if the destination resolves outside the config root, don't
    // walk up arbitrary filesystem ancestors looking for a `.gitignore`.
    if !start.starts_with(config_dir) {
        return config_dir.join(".gitignore");
    }

    let mut cursor = Some(start);
    while let Some(dir) = cursor {
        let candidate = dir.join(".gitignore");
        if candidate.exists() {
            return candidate;
        }
        if dir == config_dir {
            break;
        }
        cursor = dir.parent();
    }

    config_dir.join(".gitignore")
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::prelude::{FileWriteStr, PathChild};

    #[test]
    fn no_backup_when_file_absent() {
        let dir = assert_fs::TempDir::new().unwrap();
        let target = dir.child(".env");
        assert!(backup_existing_if_changed(target.path(), "fresh content").is_none());
        assert!(!dir.child(".env.bak").path().exists());
    }

    #[test]
    fn no_backup_when_content_unchanged() {
        let dir = assert_fs::TempDir::new().unwrap();
        let target = dir.child(".env");
        target.write_str("FOO=bar\n").unwrap();
        assert!(backup_existing_if_changed(target.path(), "FOO=bar\n").is_none());
        assert!(!dir.child(".env.bak").path().exists());
    }

    #[test]
    fn backup_written_when_content_differs() {
        let dir = assert_fs::TempDir::new().unwrap();
        let target = dir.child(".env");
        target.write_str("FOO=old\n").unwrap();

        let backup = backup_existing_if_changed(target.path(), "FOO=new\n").unwrap();
        assert_eq!(backup, dir.child(".env.bak").path());

        let backup_content = std::fs::read_to_string(&backup).unwrap();
        assert_eq!(backup_content, "FOO=old\n");
    }

    #[test]
    fn backup_overwrites_previous_backup() {
        let dir = assert_fs::TempDir::new().unwrap();
        let target = dir.child(".env");
        let prior_backup = dir.child(".env.bak");
        target.write_str("v2\n").unwrap();
        prior_backup.write_str("v0\n").unwrap();

        let backup = backup_existing_if_changed(target.path(), "v3\n").unwrap();
        let content = std::fs::read_to_string(&backup).unwrap();
        // The new backup is the v2 (current), not the stale v0.
        assert_eq!(content, "v2\n");
    }

    #[test]
    fn gitignore_appended_with_separator_when_no_trailing_newline() {
        let dir = assert_fs::TempDir::new().unwrap();
        let config_path = dir.child(".wukong.toml");
        config_path.write_str("").unwrap();
        let gitignore = dir.child(".gitignore");
        // Existing content with no trailing newline.
        gitignore.write_str("node_modules").unwrap();
        let env_path = dir.child(".env");

        let mut seen = HashSet::new();
        ensure_backup_gitignored(
            env_path.path(),
            &config_path.path().to_string_lossy(),
            &mut seen,
        );

        let content = std::fs::read_to_string(gitignore.path()).unwrap();
        // Should be `node_modules\n*.bak\n`, not `node_modules*.bak\n`.
        assert_eq!(content, "node_modules\n*.bak\n");
    }

    #[test]
    fn find_gitignore_falls_back_when_dest_outside_config_dir() {
        let outer = assert_fs::TempDir::new().unwrap();
        let config_dir = outer.child("project");
        std::fs::create_dir_all(config_dir.path()).unwrap();
        // A file living outside the project root (e.g. someone abusing `..`).
        let escaped = outer.child("escape/.env");

        let target = find_gitignore_target(escaped.path(), config_dir.path());
        // Must NOT walk up into `outer` looking for a .gitignore — must
        // fall back to the project root.
        assert_eq!(target, config_dir.child(".gitignore").path());
    }

    #[test]
    fn gitignore_created_when_missing() {
        let dir = assert_fs::TempDir::new().unwrap();
        let config_path = dir.child(".wukong.toml");
        config_path.write_str("").unwrap();
        let env_path = dir.child(".env");

        let mut seen = HashSet::new();
        ensure_backup_gitignored(
            env_path.path(),
            &config_path.path().to_string_lossy(),
            &mut seen,
        );

        let gitignore = dir.child(".gitignore");
        assert!(gitignore.path().exists());
        let content = std::fs::read_to_string(gitignore.path()).unwrap();
        assert!(content.contains(BACKUP_GITIGNORE_PATTERN));
    }

    #[test]
    fn gitignore_appended_when_pattern_missing() {
        let dir = assert_fs::TempDir::new().unwrap();
        let config_path = dir.child(".wukong.toml");
        config_path.write_str("").unwrap();
        let gitignore = dir.child(".gitignore");
        gitignore.write_str("node_modules\n").unwrap();
        let env_path = dir.child(".env");

        let mut seen = HashSet::new();
        ensure_backup_gitignored(
            env_path.path(),
            &config_path.path().to_string_lossy(),
            &mut seen,
        );

        let content = std::fs::read_to_string(gitignore.path()).unwrap();
        assert!(content.contains("node_modules"));
        assert!(content.contains(BACKUP_GITIGNORE_PATTERN));
    }

    #[test]
    fn gitignore_not_modified_when_pattern_present() {
        let dir = assert_fs::TempDir::new().unwrap();
        let config_path = dir.child(".wukong.toml");
        config_path.write_str("").unwrap();
        let gitignore = dir.child(".gitignore");
        let original = format!("node_modules\n{}\n", BACKUP_GITIGNORE_PATTERN);
        gitignore.write_str(&original).unwrap();
        let env_path = dir.child(".env");

        let mut seen = HashSet::new();
        ensure_backup_gitignored(
            env_path.path(),
            &config_path.path().to_string_lossy(),
            &mut seen,
        );

        let content = std::fs::read_to_string(gitignore.path()).unwrap();
        assert_eq!(content, original);
    }

    #[test]
    fn gitignore_only_touched_once_per_run() {
        let dir = assert_fs::TempDir::new().unwrap();
        let config_path = dir.child(".wukong.toml");
        config_path.write_str("").unwrap();
        let env_path = dir.child(".env");

        let mut seen = HashSet::new();
        ensure_backup_gitignored(
            env_path.path(),
            &config_path.path().to_string_lossy(),
            &mut seen,
        );
        // Simulate a second annotation in the same run.
        let other_path = dir.child(".env.local");
        ensure_backup_gitignored(
            other_path.path(),
            &config_path.path().to_string_lossy(),
            &mut seen,
        );

        let content = std::fs::read_to_string(dir.child(".gitignore").path()).unwrap();
        let occurrences = content.matches(BACKUP_GITIGNORE_PATTERN).count();
        assert_eq!(occurrences, 1);
    }

    #[test]
    fn find_gitignore_walks_up_to_existing() {
        let dir = assert_fs::TempDir::new().unwrap();
        let config_path = dir.child(".wukong.toml");
        config_path.write_str("").unwrap();
        let root_gitignore = dir.child(".gitignore");
        root_gitignore.write_str("").unwrap();

        // Destination is nested 2 dirs deep; should resolve to the root .gitignore.
        let nested = dir.child("priv/files/kubeconfig");
        let target = find_gitignore_target(nested.path(), dir.path());
        assert_eq!(target, root_gitignore.path());
    }

    #[test]
    fn find_gitignore_falls_back_to_config_dir() {
        let dir = assert_fs::TempDir::new().unwrap();
        let nested = dir.child("priv/files/kubeconfig");
        let target = find_gitignore_target(nested.path(), dir.path());
        assert_eq!(target, dir.child(".gitignore").path());
    }
}
