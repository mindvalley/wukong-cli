use tree_sitter::{Parser, Query, QueryCursor};

#[allow(dead_code)]
pub fn read_vault_annotation(src: &str) -> Vec<(&str, &str)> {
    // the annotation patter -> wukong.mindvalley.dev/config-secrets-location: vault:secret/path/to/secret#secret_key

    let elixir_lang = tree_sitter_elixir::language();
    let mut parser = Parser::new();
    parser.set_language(elixir_lang).unwrap();

    let tree = parser.parse(src, None).unwrap();
    println!("{}", tree.root_node().to_sexp());

    let query = Query::new(
        elixir_lang,
        r#"
(
    (comment) @comment
    .
    [
        (call target: (identifier) @identifier (arguments (string (quoted_content) @import_file)))
        (call 
            target: (identifier)
          (arguments 
            (call 
                target: (_) @file_checking 
      	        (arguments (string (quoted_content) @checked_file))))
          (do_block
			        (call 
				        target: (identifier) @identifier
				        (arguments (string (quoted_content) @import_file))))
		        )
	    (binary_operator 
		    left: (call 
			    target: (_) @file_checking 
			    (arguments (string (quoted_content) @checked_file)))
		    right: (call 
			    target: (identifier) @identifier
			    (arguments (string (quoted_content) @import_file))))
    ]
    (#eq? @file_checking "File.exists?")
    (#match? @identifier "import_config|import_config!")
    (#match? @comment "wukong.mindvalley.dev/config-secrets-location:")
)
        "#,
    )
    .unwrap();

    let mut query_cursor = QueryCursor::new();
    let all_matches = query_cursor.matches(&query, tree.root_node(), src.as_bytes());

    let comment_idx = query.capture_index_for_name("comment").unwrap();
    let import_file_idx = query.capture_index_for_name("import_file").unwrap();

    let mut annotations = vec![];
    for each in all_matches {
        let annotiation = each
            .captures
            .iter()
            .find(|c| c.index == comment_idx)
            .unwrap();

        let file_name = each
            .captures
            .iter()
            .find(|c| c.index == import_file_idx)
            .unwrap();

        annotations.push((
            annotiation.node.utf8_text(src.as_bytes()).unwrap(),
            file_name.node.utf8_text(src.as_bytes()).unwrap(),
        ));
    }

    annotations
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_vault_annotation() {
        let src = r#"
    import Config

    import_config("not_match_2.secrets.exs")

    # wukong.mindvalley.dev/config-secrets-location: vault:secret/path/to/secret#secret_key
    import_config("config/config.exs")

    # wukong.mindvalley.dev/config-secrets-location: vault:secret/abc/development#abc.secrets.exs
    if File.exists? "abc.secrets.exs" do
     import_config("abc.secrets.exs")
    end

    if File.exists? "xyz.secrets.exs" do
     # wukong.mindvalley.dev/config-secrets-location: vault:secret/xyz/development#xyz.secrets.exs
     import_config("xyz.secrets.exs")
    end

    # random comment

    # wukong.mindvalley.dev/config-secrets-location: vault:secret/osiris/development#dev.secrets.exs

    # wukong.mindvalley.dev/config-secrets-location: vault:secret/abc/development#aaa.secrets.exs
    File.exists?("aaa.secrets.exs") && import_config("aaa.secrets.exs")

    # wukong.mindvalley.dev/config-secrets-location: vault:secret/xyz/development#test.secrets.exs
    import_config("test.secrets.exs")

    #wukong.mindvalley.dev/config-secrets-location: vault:secret/abc/development#prod.secrets.exs

    import_config("prod.secrets.exs")

    import_config("not_match_2.secrets.exs")
    "#;

        let annotations = read_vault_annotation(src);
        assert_eq!(annotations.len(), 6);
        assert_eq!(annotations[0].0, "# wukong.mindvalley.dev/config-secrets-location: vault:secret/path/to/secret#secret_key");
        assert_eq!(annotations[0].1, "config/config.exs");
        assert_eq!(annotations[1].0, "# wukong.mindvalley.dev/config-secrets-location: vault:secret/abc/development#abc.secrets.exs");
        assert_eq!(annotations[1].1, "abc.secrets.exs");
        assert_eq!(annotations[2].0, "# wukong.mindvalley.dev/config-secrets-location: vault:secret/xyz/development#xyz.secrets.exs");
        assert_eq!(annotations[2].1, "xyz.secrets.exs");
        assert_eq!(annotations[3].0, "# wukong.mindvalley.dev/config-secrets-location: vault:secret/abc/development#aaa.secrets.exs");
        assert_eq!(annotations[3].1, "aaa.secrets.exs");
        assert_eq!(annotations[4].0, "# wukong.mindvalley.dev/config-secrets-location: vault:secret/xyz/development#test.secrets.exs");
        assert_eq!(annotations[4].1, "test.secrets.exs");
        assert_eq!(annotations[5].0, "#wukong.mindvalley.dev/config-secrets-location: vault:secret/abc/development#prod.secrets.exs");
        assert_eq!(annotations[5].1, "prod.secrets.exs");
    }
}
