use serde::{Deserialize, Serialize};
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultSecretAnnotation {
    pub key: String,
    pub secret_path: String,
    pub secret_name: String,
    pub destination_file: String,
}

pub fn read_vault_annotation(src: &str) -> Vec<VaultSecretAnnotation> {
    // the annotation pattern -> wukong.mindvalley.dev/config-secrets-location: vault:secret/path/to/secret#secret_key

    let elixir_lang = tree_sitter_elixir::language();
    let mut parser = Parser::new();
    parser.set_language(elixir_lang).unwrap();

    let tree = parser.parse(src, None).unwrap();

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
    (#match? @comment "\#( )*wukong.mindvalley.dev/config-secrets-location:")
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
        let annotation = each
            .captures
            .iter()
            .find(|c| c.index == comment_idx)
            .unwrap();
        let annotation_text = annotation.node.utf8_text(src.as_bytes()).unwrap();
        let annotation_part: Vec<String> = annotation_text
            .replacen('#', "", 1)
            .split(": ")
            .map(|each| each.trim().to_string())
            .collect();

        if annotation_part.len() != 2 {
            continue;
        }

        let annotation_key = annotation_part[0].clone();
        let annotation_value = annotation_part[1].clone();
        let annotation_value_part = annotation_value.split('#').collect::<Vec<&str>>();
        let annotation_secret_path = annotation_value_part[0].to_string();
        let annotation_secret_name = annotation_value_part[1].to_string();

        let file_name = each
            .captures
            .iter()
            .find(|c| c.index == import_file_idx)
            .unwrap();

        annotations.push(VaultSecretAnnotation {
            key: annotation_key,
            secret_path: annotation_secret_path,
            secret_name: annotation_secret_name,
            destination_file: file_name
                .node
                .utf8_text(src.as_bytes())
                .unwrap()
                .trim()
                .to_string(),
        });
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

    # you can use wukong.mindvalley.dev/config-secrets-location: vault:secret/location to annotate the secret path
    import_config("not_match_2.secrets.exs")
    "#;

        let annotations = read_vault_annotation(src);
        assert_eq!(annotations.len(), 6);
        assert_eq!(
            annotations[0].key,
            "wukong.mindvalley.dev/config-secrets-location".to_string(),
        );
        assert_eq!(
            annotations[0].secret_path,
            "vault:secret/path/to/secret".to_string()
        );
        assert_eq!(annotations[0].secret_name, "secret_key".to_string());
        assert_eq!(annotations[0].destination_file, "config/config.exs");

        assert_eq!(
            annotations[1].key,
            "wukong.mindvalley.dev/config-secrets-location".to_string(),
        );
        assert_eq!(
            annotations[1].secret_path,
            "vault:secret/abc/development".to_string()
        );
        assert_eq!(annotations[1].secret_name, "abc.secrets.exs".to_string());
        assert_eq!(annotations[1].destination_file, "abc.secrets.exs");

        assert_eq!(
            annotations[2].key,
            "wukong.mindvalley.dev/config-secrets-location".to_string(),
        );
        assert_eq!(
            annotations[2].secret_path,
            "vault:secret/xyz/development".to_string()
        );
        assert_eq!(annotations[2].secret_name, "xyz.secrets.exs".to_string());
        assert_eq!(annotations[2].destination_file, "xyz.secrets.exs");

        assert_eq!(
            annotations[3].key,
            "wukong.mindvalley.dev/config-secrets-location".to_string(),
        );
        assert_eq!(
            annotations[3].secret_path,
            "vault:secret/abc/development".to_string()
        );
        assert_eq!(annotations[3].secret_name, "aaa.secrets.exs".to_string());
        assert_eq!(annotations[3].destination_file, "aaa.secrets.exs");

        assert_eq!(
            annotations[4].key,
            "wukong.mindvalley.dev/config-secrets-location".to_string(),
        );
        assert_eq!(
            annotations[4].secret_path,
            "vault:secret/xyz/development".to_string()
        );
        assert_eq!(annotations[4].secret_name, "test.secrets.exs".to_string());
        assert_eq!(annotations[4].destination_file, "test.secrets.exs");

        assert_eq!(
            annotations[5].key,
            "wukong.mindvalley.dev/config-secrets-location".to_string(),
        );
        assert_eq!(
            annotations[5].secret_path,
            "vault:secret/abc/development".to_string()
        );
        assert_eq!(annotations[5].secret_name, "prod.secrets.exs".to_string());
        assert_eq!(annotations[5].destination_file, "prod.secrets.exs");
    }
}
