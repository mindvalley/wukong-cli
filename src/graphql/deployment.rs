use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelinesQuery;

impl CdPipelinesQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
        application: &str,
    ) -> Result<Response<cd_pipelines_query::ResponseData>, APIError> {
        let variables = cd_pipelines_query::Variables {
            application: application.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| match error.message.as_str() {
                "application_not_found" => Err(APIError::ResponseError {
                    code: error.message,
                    message: format!("Application `{}` not found.", application),
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
    async fn test_fetch_cd_pipeline_list_success_should_return_cd_pipeline_list() {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "cdPipelines": [
      {
        "deployedRef": null,
        "enabled": true,
        "environment": "prod",
        "lastDeployment": 1663161661001,
        "name": "pipeline-blue",
        "status": "TERMINAL",
        "version": "blue"
      },
      {
        "deployedRef": null,
        "enabled": true,
        "environment": "prod",
        "lastDeployment": null,
        "name": "pipeline-green",
        "status": null,
        "version": "green"
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

        let response = CdPipelinesQuery::fetch(&query_client, "valid_application").await;

        mock.assert();
        assert!(response.is_ok());

        let cd_pipelines = response.unwrap().data.unwrap().cd_pipelines.unwrap();
        assert_eq!(cd_pipelines.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_cd_pipeline_list_failed_with_application_not_found_error_should_return_response_error(
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
    "cdPipelines": null
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
        "cdPipelines"
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

        let response = CdPipelinesQuery::fetch(&query_client, "invalid_application").await;

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
}
