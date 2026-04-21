use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::WKCliError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
    pub role: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LayoutMap {
    pub screen_title: Option<String>,
    pub back_button: Option<Element>,
    pub tab_bar: Vec<serde_json::Value>,
    pub elements: Vec<Element>,
    pub visible_text: Vec<String>,
    pub scroll_hints: serde_json::Value,
    pub context: Option<String>,
    pub suggested_actions: Vec<String>,
}

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
