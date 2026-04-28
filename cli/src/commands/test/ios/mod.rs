use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use crate::error::{TestError, WKCliError};

use super::platform::{Element, LayoutMap, PlatformBackend};

/// The simClaw bash backend is vendored in `simclaw/` as a tree of files:
/// a `bin/sim` entry point that sources the `lib/simclaw/*.sh` helpers, and
/// a `swift/` subtree used by the inspection pipeline. We embed every file
/// at compile time and extract the whole tree to disk before dispatching
/// so the script's own `$SIM_LIB` resolution works unchanged.
macro_rules! simclaw_files {
    ($($rel:literal),+ $(,)?) => {
        &[$(($rel, include_str!(concat!("simclaw/", $rel)))),+]
    };
}

const SIMCLAW_FILES: &[(&str, &str)] = simclaw_files!(
    "bin/sim",
    "lib/simclaw/bootstrap.sh",
    "lib/simclaw/coords.sh",
    "lib/simclaw/core.sh",
    "lib/simclaw/device.sh",
    "lib/simclaw/inspect.sh",
    "lib/simclaw/layout_map.sh",
    "lib/simclaw/misc.sh",
    "lib/simclaw/nav.sh",
    "lib/simclaw/setup.sh",
    "lib/simclaw/touch.sh",
    "lib/simclaw/type.sh",
    "lib/simclaw/wait.sh",
    "lib/simclaw/wda.sh",
    "lib/simclaw/swift/pickwindow.swift",
);

const EXTRACT_SUBDIR: &str = "simclaw";
const ENTRY_SCRIPT: &str = "bin/sim";
const VERSION_MARKER: &str = ".wukong-version";

/// Upstream simClaw applies `snapshotMaxDepth=15, snapshotMaxChildren=25` when
/// creating the WDA session (see `lib/simclaw/wda.sh`). Those caps are too
/// tight for anything larger than a hello-world screen — Settings.app's root
/// alone has >25 children and the tree gets truncated before reaching the
/// cells, so `layout-map` / `find-element` / `tap-on` return empty. Post
/// permissive settings after the session is live. Remove this workaround
/// when the upstream script raises (or exposes) the caps.
const PERMISSIVE_SNAPSHOT_BODY: &str =
    r#"{"settings":{"snapshotMaxDepth":60,"snapshotMaxChildren":200}}"#;

pub struct IosBackend {
    device: Option<String>,
    source_timeout: Option<u32>,
}

impl IosBackend {
    pub fn new(device: Option<String>, source_timeout: Option<u32>) -> Self {
        Self {
            device,
            source_timeout,
        }
    }

    /// Extract the bundled simClaw tree to `~/.config/wukong/scripts/simclaw/`
    /// and return the path to its `bin/sim` entry script. A `.wukong-version`
    /// marker lets us skip the 15 writes when the on-disk tree already matches
    /// this build — upgrades still force a re-extract because the marker
    /// disagrees. Only `bin/sim` needs the executable bit; the `.sh` helpers
    /// are sourced, not invoked.
    fn script_path() -> Result<PathBuf, WKCliError> {
        let extract_err = |e: std::io::Error| {
            WKCliError::TestError(TestError::ScriptExtractionFailed(e.to_string()))
        };

        let mut root = dirs::home_dir().ok_or_else(|| {
            WKCliError::TestError(TestError::ScriptExtractionFailed(
                "could not resolve home directory".into(),
            ))
        })?;
        root.extend([".config", "wukong", "scripts", EXTRACT_SUBDIR]);
        let entry = root.join(ENTRY_SCRIPT);
        let marker = root.join(VERSION_MARKER);
        let expected_version = env!("CARGO_PKG_VERSION");

        if std::fs::read_to_string(&marker).ok().as_deref() == Some(expected_version)
            && entry.exists()
        {
            return Ok(entry);
        }

        for (rel, contents) in SIMCLAW_FILES {
            let dest = root.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(extract_err)?;
            }
            std::fs::write(&dest, contents).map_err(extract_err)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&entry, std::fs::Permissions::from_mode(0o755))
                .map_err(extract_err)?;
        }

        std::fs::write(&marker, expected_version).map_err(extract_err)?;

        Ok(entry)
    }

    fn build_command(&self, subcommand: &str, args: &[String]) -> Result<Command, WKCliError> {
        let script = Self::script_path()?;
        let mut cmd = Command::new("bash");
        cmd.arg(&script);
        if let Some(device) = &self.device {
            cmd.arg("--device").arg(device);
        }
        if let Some(t) = self.source_timeout {
            cmd.arg("--source-timeout").arg(t.to_string());
        }
        cmd.arg(subcommand);
        for a in args {
            cmd.arg(a);
        }
        Ok(cmd)
    }

    /// `run_capture` is for commands whose stdout the Rust layer parses;
    /// `run_streaming` is for commands whose output goes straight to the
    /// user. Both forward stderr directly to the terminal.
    async fn run_capture(&self, subcommand: &str, args: &[String]) -> Result<String, WKCliError> {
        let mut cmd = self.build_command(subcommand, args)?;
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        let mut child = cmd.spawn().map_err(WKCliError::Io)?;
        let mut stdout = child
            .stdout
            .take()
            .expect("stdout piped when building command");
        let mut collected = String::new();
        stdout
            .read_to_string(&mut collected)
            .await
            .map_err(WKCliError::Io)?;

        let status = child.wait().await.map_err(WKCliError::Io)?;
        if !status.success() {
            return Err(WKCliError::TestError(TestError::ScriptFailed {
                subcommand: subcommand.to_string(),
                exit_code: status.code(),
            }));
        }
        Ok(collected)
    }

    async fn run_streaming(&self, subcommand: &str, args: &[String]) -> Result<(), WKCliError> {
        let mut cmd = self.build_command(subcommand, args)?;
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        let status = cmd.status().await.map_err(WKCliError::Io)?;
        if !status.success() {
            return Err(WKCliError::TestError(TestError::ScriptFailed {
                subcommand: subcommand.to_string(),
                exit_code: status.code(),
            }));
        }
        Ok(())
    }

    async fn run_json<T: DeserializeOwned>(
        &self,
        subcommand: &str,
        args: &[String],
    ) -> Result<T, WKCliError> {
        let stdout = self.run_capture(subcommand, args).await?;
        // Some compound script commands (e.g. tap-and-wait) print progress
        // lines like "READY: <label>" on stdout before emitting the final
        // JSON payload. Slice from the first `{` or `[` so the caller's
        // deserializer sees only the structured tail.
        let json_start = stdout.find(['{', '[']);
        let payload = json_start
            .map(|i| stdout[i..].trim())
            .unwrap_or(stdout.trim());
        serde_json::from_str::<T>(payload).map_err(|e| {
            WKCliError::TestError(TestError::InvalidScriptOutput {
                subcommand: subcommand.to_string(),
                reason: e.to_string(),
            })
        })
    }
}

fn opt(value: &Option<f64>) -> Option<String> {
    value.map(|v| v.to_string())
}

/// Read the WDA session ID from whichever bootstrap cache the script wrote.
/// Upstream simClaw occasionally keys the cache by `"default"` instead of the
/// UDID, so we check both the UDID-keyed file and the `_default` fallback.
fn read_wda_session() -> Option<String> {
    let dir = std::path::Path::new("/tmp");
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("sim_bootstrap_cache_") || !name.ends_with(".json") {
            continue;
        }
        let body = std::fs::read_to_string(entry.path()).ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&body).ok()?;
        if let Some(session) = parsed.get("wda_session").and_then(|v| v.as_str()) {
            if !session.is_empty() {
                return Some(session.to_string());
            }
        }
    }
    None
}

/// Best-effort: raise WDA's snapshot caps so `layout-map` returns real trees
/// on non-trivial apps. Silently no-ops if WDA isn't reachable yet — this is
/// called as a post-setup nicety, not a hard requirement.
async fn apply_permissive_snapshot_settings(port: u16) {
    let Some(session) = read_wda_session() else {
        return;
    };
    let url = format!("http://localhost:{port}/session/{session}/appium/settings");
    let _ = reqwest::Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .body(PERMISSIVE_SNAPSHOT_BODY)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;
}

#[async_trait]
impl PlatformBackend for IosBackend {
    async fn setup(&self, app: &str, port: u16) -> Result<(), WKCliError> {
        self.run_streaming(
            "setup",
            &[app.to_string(), "--port".into(), port.to_string()],
        )
        .await?;
        apply_permissive_snapshot_settings(port).await;
        Ok(())
    }

    async fn wda_start(&self, port: u16) -> Result<(), WKCliError> {
        self.run_streaming("wda-start", &[port.to_string()]).await?;
        apply_permissive_snapshot_settings(port).await;
        Ok(())
    }

    async fn status(&self) -> Result<(), WKCliError> {
        self.run_streaming("status", &[]).await
    }

    async fn layout_map(&self) -> Result<LayoutMap, WKCliError> {
        self.run_json("layout-map", &[]).await
    }

    async fn tap(&self, x: f64, y: f64) -> Result<(), WKCliError> {
        self.run_streaming("tap", &[x.to_string(), y.to_string()])
            .await
    }

    async fn tap_element(&self, label: &str) -> Result<(), WKCliError> {
        self.run_streaming("tap-element", &[label.to_string()])
            .await
    }

    async fn tap_and_wait(
        &self,
        label: &str,
        expected: Option<&str>,
        timeout: u32,
    ) -> Result<LayoutMap, WKCliError> {
        // nav.sh reads `expected` as `${2:-}`, so an empty string is the
        // script's "skip expected" sentinel. Timeout must stay in $3.
        let expected = expected.unwrap_or("");
        self.run_json(
            "tap-and-wait",
            &[label.to_string(), expected.to_string(), timeout.to_string()],
        )
        .await
    }

    async fn swipe(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        steps: u32,
        step_ms: u32,
    ) -> Result<(), WKCliError> {
        self.run_streaming(
            "swipe",
            &[
                x1.to_string(),
                y1.to_string(),
                x2.to_string(),
                y2.to_string(),
                steps.to_string(),
                step_ms.to_string(),
            ],
        )
        .await
    }

    async fn scroll_up(
        &self,
        x: Option<f64>,
        from_y: Option<f64>,
        to_y: Option<f64>,
    ) -> Result<(), WKCliError> {
        let args: Vec<String> = [opt(&x), opt(&from_y), opt(&to_y)]
            .into_iter()
            .flatten()
            .collect();
        self.run_streaming("scroll-up", &args).await
    }

    async fn scroll_down(
        &self,
        x: Option<f64>,
        from_y: Option<f64>,
        to_y: Option<f64>,
    ) -> Result<(), WKCliError> {
        let args: Vec<String> = [opt(&x), opt(&from_y), opt(&to_y)]
            .into_iter()
            .flatten()
            .collect();
        self.run_streaming("scroll-down", &args).await
    }

    async fn scroll_to_visible(&self, label: &str, max_swipes: u32) -> Result<(), WKCliError> {
        self.run_streaming(
            "scroll-to-visible",
            &[label.to_string(), max_swipes.to_string()],
        )
        .await
    }

    async fn type_text(&self, text: &str) -> Result<(), WKCliError> {
        self.run_streaming("type", &[text.to_string()]).await
    }

    async fn wait_for(&self, label: &str, timeout: u32) -> Result<(), WKCliError> {
        self.run_streaming("wait-for", &[label.to_string(), timeout.to_string()])
            .await
    }

    async fn wait_for_stable(&self, timeout: u32) -> Result<(), WKCliError> {
        self.run_streaming("wait-for-stable", &[timeout.to_string()])
            .await
    }

    async fn screen_title(&self) -> Result<String, WKCliError> {
        let out = self.run_capture("screen-title", &[]).await?;
        Ok(out.trim().to_string())
    }

    async fn find_element(&self, label: &str) -> Result<Element, WKCliError> {
        self.run_json("find-element", &[label.to_string()]).await
    }

    async fn describe_point(&self, x: f64, y: f64) -> Result<serde_json::Value, WKCliError> {
        self.run_json("describe-point", &[x.to_string(), y.to_string()])
            .await
    }

    async fn describe(
        &self,
        depth: Option<u32>,
        interactive: bool,
    ) -> Result<serde_json::Value, WKCliError> {
        let mut args = Vec::new();
        if let Some(d) = depth {
            args.push(d.to_string());
        }
        if interactive {
            args.push("--interactive".into());
        }
        self.run_json("describe", &args).await
    }

    async fn screenshot(&self, output: &str) -> Result<(), WKCliError> {
        self.run_streaming("screenshot", &[output.to_string()])
            .await
    }

    async fn ensure_foreground_app(&self, bundle_id: Option<&str>) -> Result<(), WKCliError> {
        let args: Vec<String> = bundle_id.map(|b| vec![b.to_string()]).unwrap_or_default();
        self.run_streaming("ensure-foreground-app", &args).await
    }

    async fn cleanup(&self) -> Result<(), WKCliError> {
        self.run_streaming("cleanup", &[]).await
    }

    async fn health_check(&self) -> Result<(), WKCliError> {
        self.run_streaming("health-check", &[]).await
    }
}

#[cfg(test)]
mod tests {
    use super::SIMCLAW_FILES;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    /// Guards against re-vendoring drift: every file under `simclaw/` that
    /// the script needs at runtime must be listed in `SIMCLAW_FILES`.
    /// Upstream files not in the manifest would compile fine but fail at
    /// runtime when `bin/sim` tries to source them.
    #[test]
    fn manifest_matches_tree() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/commands/test/ios/simclaw");
        let on_disk: HashSet<String> = collect_tree(&root, &root)
            .into_iter()
            .filter(|rel| is_runtime_file(rel))
            .collect();
        let in_manifest: HashSet<String> = SIMCLAW_FILES
            .iter()
            .map(|(rel, _)| rel.to_string())
            .collect();

        let missing_from_manifest: Vec<_> = on_disk.difference(&in_manifest).collect();
        let missing_from_disk: Vec<_> = in_manifest.difference(&on_disk).collect();

        assert!(
            missing_from_manifest.is_empty() && missing_from_disk.is_empty(),
            "SIMCLAW_FILES drifted from simclaw/ tree.\n  files on disk but not in manifest: {:?}\n  files in manifest but not on disk: {:?}",
            missing_from_manifest,
            missing_from_disk,
        );
    }

    fn collect_tree(root: &Path, base: &Path) -> Vec<String> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(root).expect("read simclaw dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                out.extend(collect_tree(&path, base));
            } else {
                let rel = path
                    .strip_prefix(base)
                    .expect("child under base")
                    .to_str()
                    .expect("simclaw paths must be valid UTF-8")
                    .to_string();
                out.push(rel);
            }
        }
        out
    }

    /// Metadata files (VENDORED.md, revendor.sh, READMEs) are not runtime
    /// inputs — they must NOT be in `SIMCLAW_FILES`.
    fn is_runtime_file(rel: &str) -> bool {
        rel.starts_with("bin/") || rel.starts_with("lib/")
    }
}
