use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/changelogs.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ChangelogsQuery;

#[cfg(test)]
mod test {
    use super::*;
    use crate::{error::APIError, graphql::GQLClient};
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_fetch_changelog_list_success_should_return_changelog_list() {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

        let api_resp = r#"
{
  "data": {
    "changelogs": [
      {
        "shortHash": "ceaf8d80",
        "messageHeadline": "Y29tbWl0IDE=",
        "author": "user1@example.com",
        "url": "https://github.com/xxx/yyy/commit/ceaf8d80"
      },
      {
        "shortHash": "04768172",
        "messageHeadline": "Y29tbWl0IDI=",
        "author": "user2@example.com",
        "url": "https://github.com/xxx/yyy/commit/04768172"
      }
    ]
  }
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = gql_client
            .post_graphql::<ChangelogsQuery, _>(
                server.base_url(),
                changelogs_query::Variables {
                    application: "valid-application".to_string(),
                    namespace: "prod".to_string(),
                    version: "green".to_string(),
                    build_artifact_name: "main-build-1234".to_string(),
                },
            )
            .await;

        mock.assert();
        assert!(response.is_ok());

        let changelogs = response.unwrap().changelogs;
        assert_eq!(changelogs.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_changelog_list_failed_with_application_not_found_error_should_return_response_error(
    ) {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

        let api_resp = r#"
{
  "data": null,
  "errors": [
    {
      "locations": [
        {
          "column": 3,
          "line": 2
        }
      ],
      "message": "application_not_found",
      "path": [
        "changelogs"
      ]
    }
  ]
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = gql_client
            .post_graphql::<ChangelogsQuery, _>(
                server.base_url(),
                changelogs_query::Variables {
                    application: "invalid-application".to_string(),
                    namespace: "prod".to_string(),
                    version: "green".to_string(),
                    build_artifact_name: "main-build-1234".to_string(),
                },
            )
            .await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "application_not_found");
                assert_eq!(message, "Application `invalid-application` not found.");
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_changelog_list_failed_with_unable_to_determine_changelog_error_should_return_response_error(
    ) {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

        let api_resp = r#"
{
  "data": null,
  "errors": [
    {
      "locations": [
        {
          "column": 3,
          "line": 2
        }
      ],
      "message": "unable_to_determine_changelog",
      "path": [
        "changelogs"
      ]
    }
  ]
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let invalid_build_number = "invalid-build-1234";
        let response = gql_client
            .post_graphql::<ChangelogsQuery, _>(
                server.base_url(),
                changelogs_query::Variables {
                    application: "valid-application".to_string(),
                    namespace: "prod".to_string(),
                    version: "green".to_string(),
                    build_artifact_name: invalid_build_number.to_string(),
                },
            )
            .await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "unable_to_determine_changelog");
                assert_eq!(
                    message,
                    &format!(
                        "Unable to determine the changelog for {}.",
                        invalid_build_number
                    )
                );
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_changelog_list_failed_with_comparing_same_build_error_should_return_changelog_comparing_same_build_error(
    ) {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

        let api_resp = r#"
{
  "data": null,
  "errors": [
    {
      "locations": [
        {
          "column": 3,
          "line": 2
        }
      ],
      "message": "comparing_same_build",
      "path": [
        "changelogs"
      ]
    }
  ]
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let same_build_number = "same-build-1234";
        let response = gql_client
            .post_graphql::<ChangelogsQuery, _>(
                server.base_url(),
                changelogs_query::Variables {
                    application: "valid-application".to_string(),
                    namespace: "prod".to_string(),
                    version: "green".to_string(),
                    build_artifact_name: same_build_number.to_string(),
                },
            )
            .await;

        mock.assert();
        assert!(response.is_err());

        assert!(matches!(
            response.as_ref().unwrap_err(),
            APIError::ChangelogComparingSameBuild
        ));
    }
}
