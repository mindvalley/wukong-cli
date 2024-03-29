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
    use crate::{
        error::{APIError, WKError},
        ApiChannel, WKClient, WKConfig,
    };
    use httpmock::prelude::*;

    fn setup_wk_client(api_url: &str) -> WKClient {
        WKClient::new(WKConfig {
            api_url: api_url.to_string(),
            access_token: "test_access_token".to_string(),
            channel: ApiChannel::Stable,
        })
    }

    #[tokio::test]
    async fn test_fetch_changelog_list_success_should_return_changelog_list() {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

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

        let response = wk_client
            .fetch_changelogs("valid-application", "prod", "green", "main-build-1234")
            .await;

        mock.assert();
        assert!(response.is_ok());

        let changelogs = response.unwrap().changelogs;
        assert_eq!(changelogs.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_changelog_list_failed_with_application_not_found_error_should_return_application_not_found_error(
    ) {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

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
      "message": "Application not found in application config",
      "path": [
        "changelogs"
      ],
      "extensions": {
        "code": "application_not_found"
      }
    }
  ]
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = wk_client
            .fetch_changelogs("invalid-application", "prod", "green", "main-build-1234")
            .await;

        mock.assert();
        assert!(response.is_err());

        let error = response.unwrap_err();
        match &error {
            WKError::APIError(APIError::ApplicationNotFound) => {}
            _ => panic!("it should be returning APIError::ApplicationNotFound"),
        };

        assert_eq!(format!("{error}"), "Application not found.");
    }

    #[tokio::test]
    async fn test_fetch_changelog_list_failed_with_unable_to_determine_changelog_error_should_return_unable_to_determine_changelog_error(
    ) {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

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
      "message": "Unable to determine changelog",
      "path": [
        "changelogs"
      ],
      "extensions": {
        "code": "changelog_unable_to_determine"
      }
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
        let response = wk_client
            .fetch_changelogs("valid-application", "prod", "green", invalid_build_number)
            .await;

        mock.assert();
        assert!(response.is_err());

        let error = response.unwrap_err();
        match &error {
            WKError::APIError(APIError::UnableToDetermineChangelog) => {}
            _ => panic!("it should be returning APIError::UnableToDetermineChangelog"),
        };

        assert_eq!(
            format!("{error}"),
            format!("Unable to determine the changelog for this build.",)
        );
    }

    #[tokio::test]
    async fn test_fetch_changelog_list_failed_with_comparing_same_build_error_should_return_changelog_comparing_same_build_error(
    ) {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

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
      "message": "Changelog has same commit",
      "path": [
        "changelogs"
      ],
      "extensions": {
        "code": "changelog_same_commit"
      }
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
        let response = wk_client
            .fetch_changelogs("valid-application", "prod", "green", same_build_number)
            .await;

        mock.assert();
        assert!(response.is_err());

        let error = response.unwrap_err();
        assert!(matches!(
            error,
            WKError::APIError(APIError::ChangelogComparingSameBuild)
        ));

        assert_eq!(
            format!("{error}"),
            "The selected build number is the same as the current deployed version. So there is no changelog."
        );
    }
}
