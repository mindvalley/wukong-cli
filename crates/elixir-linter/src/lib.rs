pub mod rules;

use glob::Pattern;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::{collections::HashMap, time::Instant};

use miette::{Diagnostic, GraphicalReportHandler, NamedSource, SourceSpan};
use rules::{
    no_env_in_dev_config::NoEnvInDevConfig, no_env_in_main_config::NoEnvInMainConfig,
    use_import_config_with_file_exists_checking::UseImportConfigWithFileExistsChecking, Rule,
};
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

pub fn run() {
    let program_start = Instant::now();

    let elixir_lang: Language = tree_sitter_elixir::language();
    let mut parser = Parser::new();
    parser.set_language(elixir_lang).unwrap();

    let mut rule_executor = RuleExecutor::new();
    rule_executor.add_rule(Box::new(NoEnvInMainConfig::new(elixir_lang)));
    rule_executor.add_rule(Box::new(NoEnvInDevConfig::new(elixir_lang)));
    rule_executor.add_rule(Box::new(UseImportConfigWithFileExistsChecking::new(
        elixir_lang,
    )));

    let mut count = 0;

    let program_load_time_taken = program_start.elapsed();

    let mut overrides = OverrideBuilder::new("./");
    overrides.add("**/lib/**/*.{ex,exs}").unwrap();
    overrides.add("**/test/**/*.{ex,exs}").unwrap();
    overrides.add("**/config/**/*.{ex,exs}").unwrap();

    let mut parse_tree_map = HashMap::new();

    for entry in WalkBuilder::new("./")
        .overrides(overrides.build().unwrap())
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        count += 1;

        let file_path = entry.path().to_str().unwrap();
        let src = std::fs::read_to_string(file_path).unwrap();
        let lint_errors = rule_executor.run(
            &mut parser,
            &mut parse_tree_map,
            src.to_string(),
            &file_path,
        );

        lint_errors.iter().for_each(|lint_error| {
            let mut s = String::new();
            GraphicalReportHandler::new()
                .render_report(&mut s, lint_error)
                .unwrap();

            println!("{s}");
        });
    }

    let total_time_taken = program_start.elapsed();
    let lint_time_taken = total_time_taken - program_load_time_taken;

    println!(
        "Total time taken: {:?} ({:?} to load, {:?} running {} checks)",
        total_time_taken,
        program_load_time_taken,
        lint_time_taken,
        rule_executor.rules.len(),
    );
    println!("Total files: {}", count);
}
