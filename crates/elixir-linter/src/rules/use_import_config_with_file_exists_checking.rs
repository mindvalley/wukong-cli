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
                ((call 
                    target: (identifier)
                    (arguments (call target: (_) @file_checking 
                    (arguments (string (_)) @checked_file)))
                (do_block
                    (call 
                        target: (identifier) @identifier
                    (arguments (string (_) )@import_file)))
                ) @match_with_do_block
                    (#eq? @file_checking "File.exists?")
                    (#not-eq? @checked_file @import_file)
                    (#match? @identifier "import_config|import_config!"))

                ((binary_operator 
                    left: (call 
                        target: (_) @file_checking 
                            (arguments (string (_)) @checked_file))
                    right: (call 
                        target: (identifier) @identifier
                            (arguments (string (_) )@import_file))) @match_with_binary_operator
                    (#eq? @file_checking "File.exists?")
                    (#not-eq? @checked_file @import_file)
                    (#match? @identifier "import_config|import_config!"))
                
                ((call
                    target: (identifier) @identifier (arguments (string (_)))) @match
                    (#match? @identifier "import_config|import_config!"))
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
        let match_idx = self.query().capture_index_for_name("match").unwrap();

        let matches = all_matches
            .map(|each| {
                each.captures
                    .iter()
                    .filter(|capture| {
                        capture.index == match_with_do_block_idx
                            || capture.index == match_with_binary_operator_idx
                            || capture.index == match_idx
                    })
                    .filter(|capture| {
                        if capture.index == match_idx {
                            if let Some(parent) = capture.node.parent() {
                                if parent.kind() == "source" {
                                    return true;
                                } else {
                                    return false;
                                }
                            } else {
                                return true;
                            }
                        } else {
                            return true;
                        }
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
            .flatten()
            .collect::<Vec<LintError>>();

        matches
    }
}
