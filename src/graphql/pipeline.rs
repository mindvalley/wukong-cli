use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PipelinesQuery;

impl PipelinesQuery {
    pub async fn fetch(
        client: &QueryClient,
        application: &str,
    ) -> Result<Response<pipelines_query::ResponseData>, APIError> {
        let variables = pipelines_query::Variables {
            application: Some(application.to_string()),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| {
                if error.message == "unable_to_get_pipelines" {
                    return Err(APIError::ResponseError {
                        code: error.message,
                        message: format!(
                            "Unable to get pipelines for application `{}`.",
                            application
                        ),
                    });
                }

                Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{}", error),
                })
            })
            .await?;

        Ok(response)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize, PartialEq, Eq"
)]
pub struct PipelineQuery;

impl PipelineQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
        name: &str,
    ) -> Result<Response<pipeline_query::ResponseData>, APIError> {
        let variables = pipeline_query::Variables {
            name: name.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| {
                if error.message == "unable_to_get_pipeline" {
                    return Err(APIError::ResponseError {
                        code: error.message,
                        message: format!("Unable to get pipeline `{}`.", name),
                    });
                }

                Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{}", error),
                })
            })
            .await?;

        Ok(response)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/multi_branch_pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct MultiBranchPipelineQuery;

impl MultiBranchPipelineQuery {
    pub async fn fetch(
        client: &QueryClient,
        name: &str,
    ) -> Result<Response<multi_branch_pipeline_query::ResponseData>, APIError> {
        let variables = multi_branch_pipeline_query::Variables {
            name: name.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| {
                if error.message == "unable_to_get_pipeline" {
                    return Err(APIError::ResponseError {
                        code: error.message,
                        message: format!("Unable to get pipeline `{}`.", name),
                    });
                }

                Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{}", error),
                })
            })
            .await?;

        Ok(response)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/ci_status.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CiStatusQuery;

impl CiStatusQuery {
    pub async fn fetch(
        client: &QueryClient,
        repo_url: &str,
        branch: &str,
    ) -> Result<Response<ci_status_query::ResponseData>, APIError> {
        let variables = ci_status_query::Variables {
            repo_url: repo_url.to_string(),
            branch: branch.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |resp, error| match error.message.as_str() {
                "application_not_found" => Err(APIError::ResponseError { code: "ci_status_application_not_found".to_string(), message: 
    "Could not find the application associated with this Git repo.\n\t\t\tEither you're not in the correct working folder for your application, or there's a misconfiguration.".to_string()
                }),
                "no_builds_associated_with_this_branch" => Ok(resp),
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
    async fn test_fetch_pipeline_list_success_should_return_pipeline_list() {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "pipelines": [
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

        let response = PipelinesQuery::fetch(&query_client, "mv-platform").await;

        mock.assert();
        assert!(response.is_ok());

        let pipelines = response.unwrap().data.unwrap().pipelines;
        assert_eq!(pipelines.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_pipeline_list_failed_with_unable_to_get_pipelines_error_should_return_response_error(
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
      "message": "unable_to_get_pipelines",
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

        let response = PipelinesQuery::fetch(&query_client, "invalid_application").await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "unable_to_get_pipelines");
                assert_eq!(
                    message,
                    "Unable to get pipelines for application `invalid_application`."
                )
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_pipeline_success_should_return_pipeline() {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "pipeline": {
        "__typename": "Job",
        "lastDuration": 522303,
        "lastFailedAt": 1663844109893,
        "lastSucceededAt": null,
        "name": "mv-platform-main-branch-build"
    }
  }
}
"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = PipelineQuery::fetch(&query_client, "mv-platform-main-branch-build").await;

        mock.assert();
        assert!(response.is_ok());

        match response.unwrap().data.unwrap().pipeline.unwrap() {
            pipeline_query::PipelineQueryPipeline::Job(job) => {
                assert_eq!(job.name, "mv-platform-main-branch-build");
                assert_eq!(job.last_duration, Some(522303));
                assert_eq!(job.last_failed_at, Some(1663844109893));
                assert_eq!(job.last_succeeded_at, None);
            }
            pipeline_query::PipelineQueryPipeline::MultiBranchPipeline(_pipeline) => {
                panic!("the test shouldn't reach this line");
            }
        };
    }

    #[tokio::test]
    async fn test_fetch_pipeline_failed_with_unable_to_get_pipeline_error_should_return_response_error(
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
      "message": "unable_to_get_pipeline",
      "path": [
        "pipeline"
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

        let response = PipelineQuery::fetch(&query_client, "invalid_name").await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "unable_to_get_pipeline");
                assert_eq!(message, "Unable to get pipeline `invalid_name`.")
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_multi_branch_pipeline_success_should_return_that_pipeline() {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "multiBranchPipeline": {
      "branches": [
        {
          "lastDuration": 582271,
          "lastFailedAt": 1664267048730,
          "lastSucceededAt": 1664267841689,
          "name": "main"
        }
      ],
      "lastDuration": null,
      "lastFailedAt": 1664267048730,
      "lastSucceededAt": null,
      "pullRequests": [
        {
          "lastDuration": 1259522,
          "lastFailedAt": null,
          "lastSucceededAt": 1663063337437,
          "name": "PR-1000"
        },
        {
          "lastDuration": 1263147,
          "lastFailedAt": null,
          "lastSucceededAt": 1663063574604,
          "name": "PR-1001"
        }
      ]
    }
  }
}
"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = MultiBranchPipelineQuery::fetch(&query_client, "mv-platform-ci").await;

        mock.assert();
        assert!(response.is_ok());

        let pipeline = response
            .unwrap()
            .data
            .unwrap()
            .multi_branch_pipeline
            .unwrap();

        assert_eq!(pipeline.last_duration, None);
        assert_eq!(pipeline.last_failed_at, Some(1664267048730));
        assert_eq!(pipeline.last_succeeded_at, None);
        assert_eq!(pipeline.branches.len(), 1);
        assert_eq!(pipeline.pull_requests.len(), 2);

        let branch = pipeline.branches.first().unwrap();
        assert_eq!(branch.name, "main");
        assert_eq!(branch.last_duration, Some(582271));
        assert_eq!(branch.last_failed_at, Some(1664267048730));
        assert_eq!(branch.last_succeeded_at, Some(1664267841689));
    }

    #[tokio::test]
    async fn test_fetch_multi_branch_pipeline_with_unable_to_get_pipeline_error_should_return_response_error(
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
      "message": "unable_to_get_pipeline",
      "path": [
        "multiBranchPipeline"
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

        let response = MultiBranchPipelineQuery::fetch(&query_client, "invalid_pipeline").await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "unable_to_get_pipeline");
                assert_eq!(message, "Unable to get pipeline `invalid_pipeline`.")
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_ci_status_success_should_return_ci_status() {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "ciStatus": {
      "buildDuration": 582271,
      "buildNumber": 101,
      "buildUrl": "https://ci.mv.dev/mv-platform-ci/job/main/101/",
      "commits": [],
      "name": "main",
      "result": "SUCCESS",
      "timestamp": 1664267841689,
      "totalDuration": 582274,
      "waitDuration": 0
    }
  }
}
"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response =
            CiStatusQuery::fetch(&query_client, "https://repo.com/mv-platform-ci", "main").await;

        mock.assert();
        assert!(response.is_ok());

        let ci_status = response.unwrap().data.unwrap().ci_status.unwrap();
        assert_eq!(ci_status.name, "main");
        assert_eq!(ci_status.build_duration, Some(582271));
        assert_eq!(ci_status.build_number, 101);
        assert_eq!(
            ci_status.build_url,
            "https://ci.mv.dev/mv-platform-ci/job/main/101/"
        );
        assert_eq!(ci_status.commits.len(), 0);
        assert_eq!(ci_status.result, "SUCCESS");
        assert_eq!(ci_status.timestamp, 1664267841689);
        assert_eq!(ci_status.total_duration, Some(582274));
        assert_eq!(ci_status.wait_duration, Some(0));
    }

    #[tokio::test]
    async fn test_fetch_ci_status_failed_with_application_not_found_error_should_return_response_error(
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
        "ciStatus"
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
            CiStatusQuery::fetch(&query_client, "http://invalid_repo_url.com", "main").await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "ci_status_application_not_found");
                assert_eq!(
                    message,
                    "Could not find the application associated with this Git repo.\n\t\t\tEither you're not in the correct working folder for your application, or there's a misconfiguration."
                );
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_ci_status_failed_with_no_builds_associated_with_this_branch_error_should_return_ok_response(
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
      "ciStatus": null
  },
  "errors": [
    {
      "locations": [
        {
          "column": 3,
          "line": 2
        }
      ],
      "message": "no_builds_associated_with_this_branch",
      "path": [
        "ciStatus"
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
            CiStatusQuery::fetch(&query_client, "http://valid_repo_url.com", "main").await;

        mock.assert();
        assert!(response.is_ok());
        let ci_status = response.unwrap().data.unwrap().ci_status;
        assert!(ci_status.is_none());
    }
}
