pub mod rules;

use glob::Pattern;
use globwalk::glob;
use std::time::Instant;

use miette::{Diagnostic, GraphicalReportHandler, NamedSource, SourceSpan};
use rules::{
    no_env_in_dev_config::NoEnvInDevConfig, no_env_in_main_config::NoEnvInMainConfig,
    use_import_config_with_file_exists_checking::UseImportConfigWithFileExistsChecking, Rule,
};
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

impl RuleExecutor {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule>) -> &mut Self {
        self.rules.push(rule);
        self
    }

    pub fn run(&self, parser: &mut Parser, src: String, file_path: &str) -> Vec<LintError> {
        let lint_errors: Vec<LintError> = self
            .rules
            .iter()
            .filter(|rule| Pattern::new(rule.glob()).unwrap().matches(file_path))
            .map(|rule| {
                let parse_tree = parser.parse(&src, None).unwrap();
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

    let mut lint_time_takens = vec![];
    let mut count = 0;

    let program_load_time_taken = program_start.elapsed();

    for entry in glob("**/*.{ex,exs}").expect("Failed to read glob pattern") {
        count += 1;
        let now = Instant::now();
        if let Ok(entry) = entry {
            // println!("{:?}", entry.path().as_os_str());

            let file_path = entry.path().to_str().unwrap();
            let src = std::fs::read_to_string(file_path).unwrap();
            let lint_errors = rule_executor.run(&mut parser, src.to_string(), &file_path);

            lint_errors.iter().for_each(|lint_error| {
                let mut s = String::new();
                GraphicalReportHandler::new()
                    .render_report(&mut s, lint_error)
                    .unwrap();

                println!("{s}");
            });
        }
        lint_time_takens.push(now.elapsed());
    }

    let total_time_taken = program_start.elapsed();

    let lint_time_taken = lint_time_takens.into_iter().reduce(|a, b| a + b).unwrap();

    println!(
        "Total time taken: {:?} ({:?} to load, {:?} running {} checks)",
        total_time_taken,
        program_load_time_taken,
        lint_time_taken,
        rule_executor.rules.len(),
    );
    println!("Total files: {}", count);
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
