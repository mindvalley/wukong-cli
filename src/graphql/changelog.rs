use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};

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
        build_number: i64,
    ) -> Result<Response<changelogs_query::ResponseData>, APIError> {
        let variables = changelogs_query::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
            build_number,
        };

        let response = client
            .call_api::<Self>(variables, |_, error| match error.message.as_str() {
                "application_not_found" => Err(APIError::ResponseError {
                    code: error.message,
                    message: format!("Application `{}` not found.", application),
                }),
                "unable_to_determine_changelog" => Err(APIError::ResponseError {
                    code: error.message,
                    message: format!(
                        "Unable to determine the changelog for build-{}",
                        build_number
                    ),
                }),
                _ => Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{}", error),
                }),
            })
            .await?;

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
        "id": "ceaf8d80f1f17a4de80ca4fce655700284a30c9a",
        "message": "Y29tbWl0IDE="
      },
      {
        "id": "04768172ec0417ded2995d2e2b2a0203de49fcca",
        "message": "Y29tbWl0IDI="
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

        let response =
            ChangelogsQuery::fetch(&query_client, "valid_application", "prod", "green", 1234).await;

        mock.assert();
        assert!(response.is_ok());

        let changelogs = response.unwrap().data.unwrap().changelogs.unwrap();
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
  "data": {
    "changelogs": null
  },
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

        let response =
            ChangelogsQuery::fetch(&query_client, "invalid_application", "prod", "green", 1234)
                .await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ReqwestError(_) => panic!("it shouldn't returning ReqwestError"),
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "application_not_found");
                assert_eq!(message, "Application `invalid_application` not found.");
            }
            APIError::UnAuthenticated => panic!("it shouldn't returning UnAuthenticated"),
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
  "data": {
    "changelogs": null
  },
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

        let invalid_build_number = 1234;
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
            APIError::ReqwestError(_) => panic!("it shouldn't returning ReqwestError"),
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "unable_to_determine_changelog");
                assert_eq!(
                    message,
                    &format!(
                        "Unable to determine the changelog for build-{}",
                        invalid_build_number
                    )
                );
            }
            APIError::UnAuthenticated => panic!("it shouldn't returning UnAuthenticated"),
        }
    }
}
