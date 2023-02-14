use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};
use log::debug;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/changelogs.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ChangelogsQuery;

impl ChangelogsQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
    ) -> Result<Response<changelogs_query::ResponseData>, APIError> {
        let variables = changelogs_query::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
            build_artifact_name: build_artifact_name.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| match error.message.as_str() {
                "application_not_found" => Err(APIError::ResponseError {
                    code: error.message,
                    message: format!("Application `{application}` not found."),
                }),
                "unable_to_determine_changelog" => Err(APIError::ResponseError {
                    code: error.message,
                    message: format!(
                        "Unable to determine the changelog for {build_artifact_name}."
                    ),
                }),
                "comparing_same_build" => Err(APIError::ChangelogComparingSameBuild),
                _ => Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
                }),
            })
            .await?;

        debug!("response: {:?}", &response);

        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::graphql::QueryClientBuilder;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_fetch_changelog_list_success_should_return_changelog_list() {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

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

        let response = ChangelogsQuery::fetch(
            &query_client,
            "valid_application",
            "prod",
            "green",
            "main-build-1234",
        )
        .await;

        mock.assert();
        assert!(response.is_ok());

        let changelogs = response.unwrap().data.unwrap().changelogs;
        assert_eq!(changelogs.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_changelog_list_failed_with_application_not_found_error_should_return_response_error(
    ) {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

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

        let response = ChangelogsQuery::fetch(
            &query_client,
            "invalid_application",
            "prod",
            "green",
            "main-build-1234",
        )
        .await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "application_not_found");
                assert_eq!(message, "Application `invalid_application` not found.");
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_changelog_list_failed_with_unable_to_determine_changelog_error_should_return_response_error(
    ) {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

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
        let response = ChangelogsQuery::fetch(
            &query_client,
            "valid_application",
            "prod",
            "green",
            invalid_build_number,
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
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

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

        let invalid_build_number = "invalid-build-1234";
        let response = ChangelogsQuery::fetch(
            &query_client,
            "valid_application",
            "prod",
            "green",
            invalid_build_number,
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
