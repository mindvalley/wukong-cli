use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::WKCliError;

const MANIFEST_FILE: &str = "mv-manifest.json";

pub fn update_manifest(root: &Path, slug: &str, content_hash: &str) -> Result<(), WKCliError> {
    let manifest_path = root.join(".agents").join("skills").join(MANIFEST_FILE);

    let mut manifest: std::collections::HashMap<String, String> = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    manifest.insert(slug.to_string(), content_hash.to_string());

    let json = serde_json::to_string_pretty(&manifest).map_err(|e| {
        WKCliError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    fs::write(&manifest_path, json)?;

    Ok(())
}

/// Create a symlink from `claude_file` pointing to `agents_file`.
/// Computes a relative path dynamically instead of hardcoding depth.
/// On Windows, creates a regular file symlink (requires developer mode or elevation).
pub fn create_skill_symlink(agents_file: &Path, claude_file: &Path) -> Result<(), WKCliError> {
    let claude_parent = claude_file.parent().ok_or_else(|| {
        WKCliError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "cannot determine parent of claude skill file",
        ))
    })?;

    let relative_target = compute_relative_path(claude_parent, agents_file);

    #[cfg(unix)]
    std::os::unix::fs::symlink(&relative_target, claude_file)?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&relative_target, claude_file)?;

    Ok(())
}

fn compute_relative_path(from_dir: &Path, to_file: &Path) -> PathBuf {
    let from = normalize(from_dir);
    let to = normalize(to_file);

    let common_len = from
        .components()
        .zip(to.components())
        .take_while(|(a, b)| a == b)
        .count();

    let ups = from.components().count() - common_len;
    let mut rel = PathBuf::new();
    for _ in 0..ups {
        rel.push("..");
    }
    for component in to.components().skip(common_len) {
        rel.push(component);
    }
    rel
}

fn normalize(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for c in path.components() {
        match c {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            _ => components.push(c),
        }
    }
    components.iter().collect()
}
