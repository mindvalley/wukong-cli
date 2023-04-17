use crate::LintError;
use miette::NamedSource;
use tree_sitter::{Language, Query, QueryCursor, Tree};

use super::Rule;

pub struct UseImportConfigWithFileExistsChecking {
    query: Query,
}

impl UseImportConfigWithFileExistsChecking {
    pub fn new(lang: Language) -> Self {
        let query = Query::new(
            lang,
            r#"
            [
                (do_block
                    (call
                        target: (_) @file_checking
                        (arguments (string (_)) @checked_file))
                    (call 
                        target: (identifier) @identifier
                        (arguments (string (_) )@import_file))) @match_with_do_block
                        (#eq? @file_checking "File.exists?")
                        (#match? @identifier "import_config|import_config!")

                (binary_operator 
                    left: (call 
                        target: (_) @file_checking 
                            (arguments (string (_)) @checked_file))
                    right: (call 
                        target: (identifier) @identifier
                            (arguments (string (_) )@import_file))) @match_with_binary_operator
                        (#eq? @file_checking "File.exists?")
                        (#match? @identifier "import_config|import_config!")
            ]
            "#,
        )
        .unwrap();

        Self { query }
    }
}

impl Rule for UseImportConfigWithFileExistsChecking {
    fn name(&self) -> &'static str {
        "use_import_config_with_file_exists_checking"
    }

    fn glob(&self) -> &'static str {
        "**/config/dev.exs"
    }

    fn explain(&self) -> &'static str {
        "Considering checking the existence of the file before importing using `File.exists?/2` function."
    }

    fn advice(&self) -> &'static str {
        "Use a static value instead of reading from environment variable. Also if this is a secret, move it to the `dev.secrets.exs` instead."
    }

    fn query(&self) -> &Query {
        &self.query
    }

    fn run(&self, parse_tree: &Tree, src: String, file_path: &str) -> Vec<LintError> {
        let mut query_cursor = QueryCursor::new();
        let all_matches =
            query_cursor.matches(self.query(), parse_tree.root_node(), src.as_bytes());

        let match_with_do_block_idx = self
            .query()
            .capture_index_for_name("match_with_do_block")
            .unwrap();
        let match_with_binary_operator_idx = self
            .query()
            .capture_index_for_name("match_with_binary_operator")
            .unwrap();

        all_matches
            .flat_map(|each| {
                each.captures
                    .iter()
                    .filter(|capture| {
                        println!("{:?}", capture);
                        capture.index == match_with_do_block_idx
                            || capture.index == match_with_binary_operator_idx
                    })
                    .filter(|capture| {
                        if let Some(parent) = capture.node.parent() {
                            println!("{:?}", parent.kind());
                            return parent.kind() == "source";
                        }

                        true
                    })
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
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<LintError>>()
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

# valid
import_config "config/dev.exs"

# valid
if File.exists?("config/dev.exs") do
  import_config "config/dev.exs"
end

# valid
if File.exists?("config/a.exs") do
  import_config "a.exs"
end

# valid
File.exists?("config/dev.exs") && import_config "config/dev.exs"
# valid
File.exists?("config/a.exs") && import_config "a.exs"

test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")

# Use Jason for JSON parsing in Phoenix
config :phoenix, :json_library, Jason
        "#;

        let elixir_lang = tree_sitter_elixir::language();
        let mut parser = Parser::new();
        parser
            .set_language(elixir_lang)
            .expect("error loading elixir grammar");
        let parse_tree = parser.parse(&source, None).unwrap();
        let rule = UseImportConfigWithFileExistsChecking::new(elixir_lang);
        let lint_result = rule.run(&parse_tree, source.to_string(), "config/dev.exs");

        assert_eq!(lint_result.len(), 0);
    }

    #[test]
    fn test_glob() {
        let valid = vec![
            "config/dev.exs",
            "./config/dev.exs",
            "path/to/config/dev.exs",
            "path/to/config/to/config/dev.exs",
        ];
        let invalid = vec![
            "config/config.exs",
            "lib/config.ex",
            "path/to/config/test.exs",
        ];

        let elixir_lang = tree_sitter_elixir::language();
        let rule = UseImportConfigWithFileExistsChecking::new(elixir_lang);
        let pattern = Pattern::new(rule.glob()).unwrap();

        valid.iter().for_each(|each| {
            assert!(pattern.matches(each));
        });
        invalid.iter().for_each(|each| {
            assert!(!pattern.matches(each));
        });
    }
}
