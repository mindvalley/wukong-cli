use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::WKCliError;

// The JSON schemas from `layout-map`, `find-element`, `describe`, and
// `describe-point` are owned by the simClaw team and haven't been frozen
// into a stable contract yet. We pass the JSON through unchanged so the
// consumer (pipe to `jq`, `from json` in nushell, etc.) can work with
// whatever shape the script produces today. Tighten to typed structs
// once the schema is contracted in the simClaw repo.
pub type LayoutMap = serde_json::Value;
pub type Element = serde_json::Value;

#[async_trait]
pub trait PlatformBackend: Send + Sync {
    async fn setup(&self, app: &str, port: u16) -> Result<(), WKCliError>;
    async fn wda_start(&self, port: u16) -> Result<(), WKCliError>;
    async fn status(&self) -> Result<(), WKCliError>;
    async fn layout_map(&self) -> Result<LayoutMap, WKCliError>;
    async fn tap(&self, x: f64, y: f64) -> Result<(), WKCliError>;
    async fn tap_element(&self, label: &str) -> Result<(), WKCliError>;
    async fn tap_and_wait(
        &self,
        label: &str,
        expected: Option<&str>,
        timeout: u32,
    ) -> Result<LayoutMap, WKCliError>;
    async fn swipe(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        steps: u32,
        step_ms: u32,
    ) -> Result<(), WKCliError>;
    async fn scroll_up(
        &self,
        x: Option<f64>,
        from_y: Option<f64>,
        to_y: Option<f64>,
    ) -> Result<(), WKCliError>;
    async fn scroll_down(
        &self,
        x: Option<f64>,
        from_y: Option<f64>,
        to_y: Option<f64>,
    ) -> Result<(), WKCliError>;
    async fn scroll_to_visible(&self, label: &str, max_swipes: u32) -> Result<(), WKCliError>;
    async fn type_text(&self, text: &str) -> Result<(), WKCliError>;
    async fn wait_for(&self, label: &str, timeout: u32) -> Result<(), WKCliError>;
    async fn wait_for_stable(&self, timeout: u32) -> Result<(), WKCliError>;
    async fn screen_title(&self) -> Result<String, WKCliError>;
    async fn find_element(&self, label: &str) -> Result<Element, WKCliError>;
    async fn describe_point(&self, x: f64, y: f64) -> Result<serde_json::Value, WKCliError>;
    async fn describe(
        &self,
        depth: Option<u32>,
        interactive: bool,
    ) -> Result<serde_json::Value, WKCliError>;
    async fn screenshot(&self, output: &str) -> Result<(), WKCliError>;
    async fn ensure_foreground_app(&self, bundle_id: Option<&str>) -> Result<(), WKCliError>;
    async fn cleanup(&self) -> Result<(), WKCliError>;
    async fn health_check(&self) -> Result<(), WKCliError>;
}
