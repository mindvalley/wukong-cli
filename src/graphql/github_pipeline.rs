use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/github_pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct GithubPipelinesQuery;

impl GithubPipelinesQuery {
    pub async fn fetch(
        client: &mut QueryClient,
        application: &str,
    ) -> Result<Response<github_pipelines_query::ResponseData>, APIError> {
        let variables = github_pipelines_query::Variables {
            application: Some(application.to_string()),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| {
                if error.message == "unable_to_get_github_pipelines" {
                    return Err(APIError::ResponseError {
                        code: error.message,
                        message: format!(
                            "Unable to get github pipelines for application `{application}`."
                        ),
                    });
                }

                Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
                })
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
    async fn test_fetch_github_pipeline_list_success_should_return_pipeline_list() {
        let server = MockServer::start();
        let mut query_client = QueryClientBuilder::default()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "githubPipelines": [
      {
        "__typename": "MultiBranchPipeline",
        "lastDuration": null,
        "lastFailedAt": null,
        "lastSucceededAt": null,
        "name": "mv-platform-ci"
      },
      {
        "__typename": "Job",
        "lastDuration": 522303,
        "lastFailedAt": 1663844109893,
        "lastSucceededAt": 1664266988871,
        "name": "mv-platform-prod-main-branch-build"
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

        let response = GithubPipelinesQuery::fetch(&mut query_client, "mv-platform").await;

        mock.assert();
        assert!(response.is_ok());

        // Print respoinse
        println!("{:#?}", response);

        let github_pipeline = response.unwrap().data.unwrap().github_pipelines;
        assert_eq!(
            github_pipeline
                .expect("Failed to get github pipelines")
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn test_fetch_github_pipeline_list_failed_with_unable_to_get_pipelines_error_should_return_response_error(
    ) {
        let server = MockServer::start();
        let mut query_client = QueryClientBuilder::default()
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
      "message": "unable_to_get_github_pipelines",
      "path": [
        "pipelines"
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

        let response = GithubPipelinesQuery::fetch(&mut query_client, "invalid_application").await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "unable_to_get_github_pipelines");
                assert_eq!(
                    message,
                    "Unable to get github pipelines for application `invalid_application`."
                )
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }
}
