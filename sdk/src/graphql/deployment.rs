use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelinesQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelineQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipeline_status.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelineStatusQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipeline_for_rollback.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelineForRollbackQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/execute_cd_pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ExecuteCdPipeline;

#[cfg(test)]
mod test {
    use crate::{
        error::{APIError, WKError},
        ApiChannel, WKClient, WKConfig,
    };

    use base64::Engine;
    use httpmock::prelude::*;

    fn setup_wk_client(api_url: &str) -> WKClient {
        WKClient::new(WKConfig {
            api_url: api_url.to_string(),
            access_token: "test_access_token".to_string(),
            channel: ApiChannel::Stable,
        })
    }

    #[tokio::test]
    async fn test_fetch_cd_pipeline_list_success_should_return_cd_pipeline_list() {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

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

        let response = wk_client.fetch_cd_pipelines("valid-application").await;

        mock.assert();
        assert!(response.is_ok());

        let cd_pipelines = response.unwrap().cd_pipelines;
        assert_eq!(cd_pipelines.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_cd_pipeline_list_failed_with_application_not_found_error_should_return_application_not_found_error(
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

        let response = wk_client.fetch_cd_pipelines("invalid-application").await;

        mock.assert();
        assert!(response.is_err());

        let error = response.unwrap_err();
        match &error {
            WKError::APIError(APIError::ApplicationNotFound) => {}
            _ => panic!("it should be returning APIError::ApplicationNotFound",),
        };

        assert_eq!(format!("{error}"), "Application not found.");
    }

    #[tokio::test]
    async fn test_fetch_cd_pipeline_for_rollback_success_should_return_cd_pipeline() {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

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

        let response = wk_client
            .fetch_previous_cd_pipeline_build("valid-application", "prod", "green")
            .await;

        mock.assert();
        assert!(response.is_ok());

        let cd_pipeline = response.unwrap().cd_pipeline.unwrap();
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
        let wk_client = setup_wk_client(&server.base_url());

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

        let response = wk_client
            .deploy_cd_pipeline_build(
                "valid-application",
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

        let deployment_url = response.unwrap().execute_cd_pipeline.url;
        assert_eq!(deployment_url, "https://cd_pipeline_deployment_url.com");
    }

    #[tokio::test]
    async fn test_execute_cd_pipeline_list_failed_with_deploy_for_this_build_is_currently_running_error_should_return_duplicate_deployment_error(
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

        let response = wk_client
            .deploy_cd_pipeline_build(
                "valid-application",
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

        let error = response.unwrap_err();
        assert!(matches!(
            error,
            WKError::APIError(APIError::DuplicatedDeployment)
        ));

        assert_eq!(
            format!("{error}"),
            "Cannot submit this deployment request, since there is another running deployment with the same arguments is running on Spinnaker.\nYou can wait a few minutes and submit the deployment again."
        );
    }
}
