pub mod rules;

use glob::Pattern;
use miette::{Diagnostic, NamedSource, SourceSpan};
use rules::{
    no_env_in_dev_config::NoEnvInDevConfig, no_env_in_main_config::NoEnvInMainConfig,
    use_import_config_with_file_exists_checking::UseImportConfigWithFileExistsChecking, Rule,
};
use std::{collections::HashMap, path::PathBuf};
use tree_sitter::{Language, Parser, Tree};

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

impl RuleExecutor {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule>) -> &mut Self {
        self.rules.push(rule);
        self
    }

    pub fn run(
        &self,
        parser: &mut Parser,
        parse_tree_map: &mut HashMap<String, Tree>,
        src: String,
        file_path: &str,
    ) -> Vec<LintError> {
        let lint_errors: Vec<LintError> = self
            .rules
            .iter()
            .filter(|rule| Pattern::new(rule.glob()).unwrap().matches(file_path))
            .map(|rule| {
                // cache the parse tree
                let parse_tree = match parse_tree_map.get(file_path) {
                    Some(tree) => tree,
                    None => {
                        let parse_tree = parser.parse(&src, None).unwrap();
                        parse_tree_map.insert(file_path.to_string(), parse_tree);
                        parse_tree_map.get(file_path).unwrap()
                    }
                };
                rule.run(&parse_tree, src.clone(), file_path)
            })
            .flatten()
            .collect();

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
    parser: Parser,
    parse_tree_map: HashMap<String, Tree>,
    executor: RuleExecutor,
}

impl Linter {
    pub fn new(rule: LintRule) -> Self {
        let elixir_lang: Language = tree_sitter_elixir::language();

        let mut parser = Parser::new();
        parser
            .set_language(elixir_lang)
            .expect("error loading elixir grammar");

        let parse_tree_map = HashMap::new();

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
            parser,
            parse_tree_map,
            executor: rule_executor,
        }
    }

    pub fn run(&mut self, path: &PathBuf) -> Vec<LintError> {
        let mut output = vec![];
        if let Ok(true) = path.try_exists() {
            output = self.executor.run(
                &mut self.parser,
                &mut self.parse_tree_map,
                std::fs::read_to_string(path).unwrap(),
                path.to_str().unwrap(),
            )
        }

        output
    }
}
