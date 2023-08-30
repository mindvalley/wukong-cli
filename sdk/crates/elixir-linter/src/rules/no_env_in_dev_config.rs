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
            .flat_map(|each_match| {
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
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use glob::Pattern;
    use tree_sitter::Parser;

    #[test]
    fn test_lint_check() {
        let source = r#"
use Mix.Config

System.get_env("API_KEY")
System.fetch_env("API_SECRET")
System.fetch_env!("API_TOKEN")

test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")

# Use Jason for JSON parsing in Phoenix
config :phoenix, :json_library, Jason
        "#;

        let elixir_lang = tree_sitter_elixir::language();
        let mut parser = Parser::new();
        parser
            .set_language(elixir_lang)
            .expect("error loading elixir grammar");
        let parse_tree = parser.parse(source, None).unwrap();
        let rule = NoEnvInDevConfig::new(elixir_lang);
        let lint_result = rule.run(&parse_tree, source.to_string(), "config/dev.exs");

        assert_eq!(lint_result.len(), 4);
    }

    #[test]
    fn test_glob() {
        let valid = vec![
            "config/dev.exs",
            "./config/dev.exs",
            "path/to/config/dev.exs",
        ];
        let invalid = vec![
            "config/config.exs",
            "lib/config.ex",
            "path/to/config/test.exs",
        ];

        let elixir_lang = tree_sitter_elixir::language();
        let rule = NoEnvInDevConfig::new(elixir_lang);
        let pattern = Pattern::new(rule.glob()).unwrap();

        valid.iter().for_each(|each| {
            assert!(pattern.matches(each));
        });
        invalid.iter().for_each(|each| {
            assert!(!pattern.matches(each));
        });
    }
}
