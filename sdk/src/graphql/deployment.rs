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
        client: &mut QueryClient,
        application: &str,
    ) -> Result<Response<cd_pipelines_query::ResponseData>, APIError> {
        let variables = cd_pipelines_query::Variables {
            application: application.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| match error.message.as_str() {
                "application_not_found" => Err(APIError::ResponseError {
                    code: error.message,
                    message: format!("Application `{application}` not found."),
                }),
                _ => Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
                }),
            })
            .await?;

        Ok(response)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelineQuery;

impl CdPipelineQuery {
    pub(crate) async fn fetch(
        client: &mut QueryClient,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<cd_pipeline_query::ResponseData>, APIError> {
        let variables = cd_pipeline_query::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| match error.message.as_str() {
                "application_not_found" => Err(APIError::ResponseError {
                    code: error.message,
                    message: format!("Application `{application}` not found."),
                }),
                _ => Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
                }),
            })
            .await?;

        Ok(response)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipeline_for_rollback.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelineForRollbackQuery;

impl CdPipelineForRollbackQuery {
    pub(crate) async fn fetch(
        client: &mut QueryClient,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<cd_pipeline_for_rollback_query::ResponseData>, APIError> {
        let variables = cd_pipeline_for_rollback_query::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| match error.message.as_str() {
                "application_not_found" => Err(APIError::ResponseError {
                    code: error.message,
                    message: format!("Application `{application}` not found."),
                }),
                _ => Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
                }),
            })
            .await?;

        Ok(response)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/execute_cd_pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ExecuteCdPipeline;

impl ExecuteCdPipeline {
    pub(crate) async fn mutate(
        client: &mut QueryClient,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
        changelogs: Option<String>,
        send_to_slack: bool,
    ) -> Result<Response<execute_cd_pipeline::ResponseData>, APIError> {
        let variables = execute_cd_pipeline::Variables {
            application: application.to_string(),
            build_number: 0,
            build_artifact_name: Some(build_artifact_name.to_string()),
            namespace: namespace.to_string(),
            version: version.to_string(),
            changelogs,
            send_to_slack,
        };

        let response = client
            .call_api::<Self>(variables, |_, error| match error.message.as_str() {
                "application_not_found" => Err(APIError::ResponseError {
                    code: error.message,
                    message: format!("Application `{application}` not found."),
                }),
                "deploy_for_this_build_is_currently_running" => Err(APIError::ResponseError {
                    code: error.message,
                    message: "Cannot submit this deployment request, since there is another running deployment with the same arguments is running on Spinnaker.\nYou can wait a few minutes and submit the deployment again.".to_string()
                }),
                _ => Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
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
    use base64::Engine;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_fetch_cd_pipeline_list_success_should_return_cd_pipeline_list() {
        let server = MockServer::start();
        let mut query_client = QueryClientBuilder::default()
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
        "version": "blue",
        "buildArtifact": "master-build-250"
      },
      {
        "deployedRef": null,
        "enabled": true,
        "environment": "prod",
        "lastDeployment": null,
        "name": "pipeline-green",
        "status": null,
        "version": "green",
        "buildArtifact": "master-build-1235"
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

        let response = CdPipelinesQuery::fetch(&mut query_client, "valid_application").await;

        mock.assert();
        assert!(response.is_ok());

        let cd_pipelines = response.unwrap().data.unwrap().cd_pipelines;
        assert_eq!(cd_pipelines.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_cd_pipeline_list_failed_with_application_not_found_error_should_return_response_error(
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

        let response = CdPipelinesQuery::fetch(&mut query_client, "invalid_application").await;

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
    async fn test_fetch_cd_pipeline_for_rollback_success_should_return_cd_pipeline() {
        let server = MockServer::start();
        let mut query_client = QueryClientBuilder::default()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "cdPipeline": {
      "buildArtifact": "main-build-10",
      "deployedRef": "d70dddc743d428f8de97610f27b75723992cbec4",
      "enabled": true,
      "environment": "prod",
      "lastDeployment": 1675324454720,
      "name": "valid-application-deployment-green",
      "previousDeployedArtifacts": [
        "main-build-10"
      ],
      "status": "SUCCEEDED",
      "version": "green"
    }
  }
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = CdPipelineForRollbackQuery::fetch(
            &mut query_client,
            "valid_application",
            "prod",
            "green",
        )
        .await;

        mock.assert();
        assert!(response.is_ok());

        let cd_pipeline = response.unwrap().data.unwrap().cd_pipeline.unwrap();
        let previous_deployed_artifacts = cd_pipeline.previous_deployed_artifacts;
        assert_eq!(previous_deployed_artifacts.len(), 1);
        assert_eq!(
            previous_deployed_artifacts.first().unwrap(),
            "main-build-10"
        );
    }

    #[tokio::test]
    async fn test_execute_cd_pipeline_success_should_return_deployment_url() {
        let server = MockServer::start();
        let mut query_client = QueryClientBuilder::default()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "executeCdPipeline" : {
      "url": "https://cd_pipeline_deployment_url.com"
    }
  }
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = ExecuteCdPipeline::mutate(
            &mut query_client,
            "valid_application",
            "prod",
            "green",
            "main-build-100",
            Some(
                base64::engine::general_purpose::STANDARD
                    .encode("This is a changelog.\n\nThis is a new changelog.\n"),
            ),
            true,
        )
        .await;

        mock.assert();
        assert!(response.is_ok());

        let deployment_url = response.unwrap().data.unwrap().execute_cd_pipeline.url;
        assert_eq!(deployment_url, "https://cd_pipeline_deployment_url.com");
    }

    #[tokio::test]
    async fn test_execute_cd_pipeline_list_failed_with_deploy_for_this_build_is_currently_running_error_should_return_response_error(
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
      "message": "deploy_for_this_build_is_currently_running",
      "path": [
        "executeCdPipeline"
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

        let response = ExecuteCdPipeline::mutate(
            &mut query_client,
            "valid_application",
            "prod",
            "green",
            "main-build-100",
            Some(
                base64::engine::general_purpose::STANDARD
                    .encode("This is a changelog.\n\nThis is a new changelog.\n"),
            ),
            true,
        )
        .await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "deploy_for_this_build_is_currently_running");
                assert_eq!(message, "Cannot submit this deployment request, since there is another running deployment with the same arguments is running on Spinnaker.\nYou can wait a few minutes and submit the deployment again.");
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }
}