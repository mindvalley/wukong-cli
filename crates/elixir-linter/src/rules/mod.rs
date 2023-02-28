pub mod no_env_in_dev_config;
pub mod no_env_in_main_config;
pub mod use_import_config_with_file_exists_checking;

use crate::LintError;
use tree_sitter::{Query, Tree};

/// Metadata containing information about a lint rule
pub struct RuleMetadata {
    pub name: &'static str,
    pub description: &'static str,
    /// whether the rule is recommended to be enabled by default
    pub recommended: bool,
    /// if the rule is deprecated, this field will contain the reason
    pub deprecated: Option<&'static str>,
}

impl RuleMetadata {
    pub fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            recommended: false,
            deprecated: None,
        }
    }

    pub fn recommended(mut self, recommended: bool) -> Self {
        self.recommended = recommended;
        self
    }

    pub fn deprecated(mut self, reason: &'static str) -> Self {
        self.deprecated = Some(reason);
        self
    }
}

pub trait Rule {
    fn name(&self) -> &'static str;
    fn glob(&self) -> &'static str;
    fn explain(&self) -> &'static str;
    fn advice(&self) -> &'static str;
    fn query(&self) -> &Query;
    fn run(&self, parse_tree: &Tree, src: String, file_path: &str) -> Vec<LintError>;
}
