pub mod rules;

use glob::Pattern;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
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

        // for result in R::run(&ctx) {
        //     let text_range =
        //         R::text_range(&ctx, &result).unwrap_or_else(|| params.query.text_range());
        //
        //     R::suppressed_nodes(&ctx, &result, &mut state.suppressions);
        //
        //     let signal = Box::new(RuleSignal::<R>::new(
        //         params.root,
        //         query_result.clone(),
        //         result,
        //         params.services,
        //         params.apply_suppression_comment,
        //     ));
        //
        //     params.signal_queue.push(SignalEntry {
        //         signal,
        //         rule: RuleKey::rule::<R>(),
        //         text_range,
        //     });
        // }
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
    language: Language,
    executor: RuleExecutor,
}

#[derive(Debug)]
pub struct LintReport {
    pub total_file_count: u32,
    pub total_checks: u32,
    pub report: Vec<LintError>,
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
            language: elixir_lang,
            executor: rule_executor,
        }
    }

    pub fn run(&self, path: &PathBuf) -> LintReport {
        let mut parser = Parser::new();
        parser
            .set_language(self.language)
            .expect("error loading elixir grammar");

        let mut count = 0;

        let mut overrides = OverrideBuilder::new(path);
        overrides.add("**/lib/**/*.{ex,exs}").unwrap();
        overrides.add("**/test/**/*.{ex,exs}").unwrap();
        overrides.add("**/config/**/*.{ex,exs}").unwrap();

        let mut parse_tree_map = HashMap::new();
        let mut all_lint_errors = vec![];

        for entry in WalkBuilder::new(path)
            .overrides(overrides.build().unwrap())
            .build()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            count += 1;

            let file_path = entry.path().to_str().unwrap();
            let src = std::fs::read_to_string(file_path).unwrap();
            let lint_errors = self.executor.run(
                &mut parser,
                &mut parse_tree_map,
                src.to_string(),
                &file_path,
            );

            all_lint_errors.extend(lint_errors);
        }

        LintReport {
            total_file_count: count,
            total_checks: self.executor.rules.len() as u32,
            report: all_lint_errors,
        }
    }
}
