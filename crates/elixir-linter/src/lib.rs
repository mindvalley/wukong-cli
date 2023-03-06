pub mod rules;

use glob::Pattern;
use miette::{Diagnostic, NamedSource, SourceSpan};
use rules::{
    no_env_in_dev_config::NoEnvInDevConfig, no_env_in_main_config::NoEnvInMainConfig,
    use_import_config_with_file_exists_checking::UseImportConfigWithFileExistsChecking, Rule,
};
use std::path::PathBuf;
use tree_sitter::{Language, Parser};

#[derive(thiserror::Error, Debug, Diagnostic)]
#[error("{name}")]
pub struct LintError {
    name: &'static str,

    #[source_code]
    src: NamedSource,

    #[label("{kind}")]
    span: SourceSpan,

    kind: &'static str,

    #[help]
    advice: &'static str,
}

pub struct RuleExecutor {
    rules: Vec<Box<dyn Rule>>,
}

impl Default for RuleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleExecutor {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule>) -> &mut Self {
        self.rules.push(rule);
        self
    }

    pub fn run(&self, parser: &mut Parser, src: String, file_path: &str) -> Vec<LintError> {
        let checks = self
            .rules
            .iter()
            .filter(|rule| Pattern::new(rule.glob()).unwrap().matches(file_path))
            .collect::<Vec<&Box<dyn Rule>>>();

        let mut lint_errors: Vec<LintError> = vec![];

        // we only parse the file if there are rules that need to be checked
        // and the file is only parsed once
        if !checks.is_empty() {
            let parse_tree = parser.parse(&src, None).unwrap();
            lint_errors = checks
                .iter()
                .flat_map(|rule| rule.run(&parse_tree, src.clone(), file_path))
                .collect();
        }

        lint_errors
    }
}

pub enum LintRule {
    All,
    Custom(Vec<AvailableRule>),
}

pub enum AvailableRule {
    NoEnvInMainConfig,
    NoEnvInDevConfig,
    UseImportConfigWithFileExistsChecking,
}

pub struct Linter {
    executor: RuleExecutor,
}

impl Linter {
    pub fn new(rule: LintRule) -> Self {
        let elixir_lang: Language = tree_sitter_elixir::language();

        let mut rule_executor = RuleExecutor::new();
        match rule {
            LintRule::All => {
                rule_executor.add_rule(Box::new(NoEnvInMainConfig::new(elixir_lang)));
                rule_executor.add_rule(Box::new(NoEnvInDevConfig::new(elixir_lang)));
                rule_executor.add_rule(Box::new(UseImportConfigWithFileExistsChecking::new(
                    elixir_lang,
                )));
            }
            LintRule::Custom(rules) => rules.iter().for_each(|rule| match rule {
                AvailableRule::NoEnvInMainConfig => {
                    rule_executor.add_rule(Box::new(NoEnvInMainConfig::new(elixir_lang)));
                }
                AvailableRule::NoEnvInDevConfig => {
                    rule_executor.add_rule(Box::new(NoEnvInDevConfig::new(elixir_lang)));
                }
                AvailableRule::UseImportConfigWithFileExistsChecking => {
                    rule_executor.add_rule(Box::new(UseImportConfigWithFileExistsChecking::new(
                        elixir_lang,
                    )));
                }
            }),
        }

        Self {
            executor: rule_executor,
        }
    }

    pub fn run(&self, path: &PathBuf) -> Vec<LintError> {
        let elixir_lang: Language = tree_sitter_elixir::language();

        let mut parser = Parser::new();
        parser
            .set_language(elixir_lang)
            .expect("error loading elixir grammar");

        let mut output = vec![];
        if let Ok(true) = path.try_exists() {
            output = self.executor.run(
                &mut parser,
                std::fs::read_to_string(path).unwrap(),
                path.to_str().unwrap(),
            )
        }

        output
    }
}
