use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipeline_github.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelineGithubQuery;

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
    async fn test_fetch_cd_pipeline_github_success_should_return_pipeline_with_github_deployment() {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

        let api_resp = r#"
{
  "data": {
    "cdPipeline": {
      "buildArtifact": "main-build-42",
      "deployedRef": "fa5dbe6dfd0ec5764e909547ee17e0ac8a297e9b",
      "enabled": true,
      "environment": "staging",
      "githubBuilds": [
        {
          "buildArtifactName": "gh-main-build-9",
          "buildBranch": "main",
          "buildDuration": 0,
          "buildNumber": 9,
          "buildUrl": "https://github.com/mindvalley/wukong-ci-mock-app/actions/runs/5819145597",
          "commits": [
            {
              "author": "fadhil.luqman@gmail.com",
              "id": "8fbd0578198e9130fe85f3c43d8ad78b2ca74c5c",
              "message": "Merge pull request #36 from mindvalley/feature/get-branch-name\n\nWe get branch name from GITHUB_HEAD_REF",
              "messageHeadline": "Merge pull request #36 from mindvalley/feature/get-branch-name"
            }
          ],
          "name": "Elixir Staging Build",
          "result": "success",
          "timestamp": 1691658407,
          "totalDuration": 90,
          "waitDuration": 0
        }
      ],
      "lastDeployment": 1692689488322,
      "lastSuccessfullyDeployedArtifact": "main-build-42",
      "name": "mv-stg-wukong-ci-mock-deployment-green",
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
            .fetch_cd_pipeline_github("valid-application", "staging", "blue")
            .await;

        mock.assert();
        assert!(response.is_ok());

        let pipeline = response.unwrap().cd_pipeline.unwrap();

        assert!(pipeline.github_builds.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_cd_pipeline_github_failed_with_github_workflow_not_found_error_should_return_unable_to_get_pipelines_error(
    ) {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

        let api_resp = r#"
{
  "data": {
    "cdPipeline": null
  },
  "errors": [
    {
      "extensions": {
        "code": "github_workflow_not_found"
      },
      "locations": [
        {
          "column": 5,
          "line": 12
        }
      ],
      "message": "Unable to get workflow",
      "path": [
        "cdPipeline",
        "githubBuilds"
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
            .fetch_cd_pipeline_github("github-pipeline-not-setup", "staging", "blue")
            .await;

        mock.assert();
        assert!(response.is_err());

        let error = response.unwrap_err();

        match &error {
            WKError::APIError(APIError::ResponseError {
                message: _message,
                code,
            }) => {
                assert_eq!(code, "Unable to get workflow");
            }
            _ => panic!("it should be returning APIError::UnableToGetPipelines"),
        };

        assert_eq!(
            format!("{error}"),
            "API Response Error: cdPipeline/githubBuilds:12:5: Unable to get workflow"
        );
    }
}
