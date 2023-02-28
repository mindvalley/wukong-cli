use super::Rule;
use crate::LintError;
use miette::NamedSource;
use tree_sitter::{Language, Query, QueryCursor};

pub struct NoEnvInDevConfig {
    query: Query,
}

impl NoEnvInDevConfig {
    pub fn new(lang: Language) -> Self {
        let query = Query::new(
            lang,
            r#"
                ((call
                    target:
                        (dot
                            left:
                                (alias)
                            right:
                                (identifier) @identifier
                        ) (arguments (string (quoted_content)))) @match
                (#match? @identifier "get_env|fetch_env|fetch_env!"))
                "#,
        )
        .unwrap();

        Self { query }
    }
}

impl Rule for NoEnvInDevConfig {
    fn name(&self) -> &'static str {
        "no_env_in_dev_config"
    }

    fn glob(&self) -> &'static str {
        "**/config/dev.exs"
    }

    fn explain(&self) -> &'static str {
        "Dev config must not contains any environment variables"
    }

    fn advice(&self) -> &'static str {
        "Use a static value instead of reading from environment variable. Also if this is a secret, move it to the `dev.secrets.exs` instead."
    }

    fn query(&self) -> &tree_sitter::Query {
        &self.query
    }

    fn run(&self, parse_tree: &tree_sitter::Tree, src: String, file_path: &str) -> Vec<LintError> {
        let mut query_cursor = QueryCursor::new();
        let all_matches =
            query_cursor.matches(self.query(), parse_tree.root_node(), src.as_bytes());

        let match_idx = self.query().capture_index_for_name("match").unwrap();

        all_matches
            .map(|each_match| {
                each_match
                    .captures
                    .iter()
                    .filter(|c| c.index == match_idx)
                    .map(|capture| {
                        let range = capture.node.range();

                        let start = miette::SourceOffset::from_location(
                            &src,
                            range.start_point.row + 1,
                            range.start_point.column + 1,
                        );
                        let end = miette::SourceOffset::from_location(
                            &src,
                            range.end_point.row + 1,
                            range.end_point.column + 1,
                        );

                        LintError {
                            name: self.name(),
                            src: NamedSource::new(file_path, src.clone()),
                            span: (
                                start,
                                miette::SourceOffset::from(end.offset() - start.offset()),
                            )
                                .into(),
                            kind: self.explain(),
                            advice: self.advice(),
                        }
                    })
                    .collect::<Vec<LintError>>()
            })
            .flatten()
            .collect()
    }
}
