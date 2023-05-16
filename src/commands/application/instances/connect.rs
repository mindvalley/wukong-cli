use crate::{commands::Context, error::CliError};

// use crate::{
//     commands::Context, error::CliError, graphql::QueryClientBuilder,
//     loader::new_spinner_progress_bar,
// };
// use tokio::time::sleep;
//
pub async fn handle_connect(_context: Context, _name: &str) -> Result<bool, CliError> {
    //     let progress_bar = new_spinner_progress_bar();
    //     progress_bar.set_message("Checking your permission to connect to the remote instance...");
    //     if has_permission().await {
    //         progress_bar.finish_and_clear();
    //         eprintln!("Checking your permission to connect to the remote instance...✅");
    //     } else {
    //         progress_bar.finish_and_clear();
    //         eprintln!("You don't have permission to connect to this instance.");
    //         eprintln!("Please check with your team manager to get approval first.");
    //         todo!();
    //     }
    //
    //     let fetching_progress_bar = new_spinner_progress_bar();
    //     fetching_progress_bar.set_message(
    //         "Listing running instances of the application mv-wukong-ci-mock on namespace production...",
    //     );
    //
    //     // Calling API ...
    //     let client = QueryClientBuilder::default()
    //         .with_access_token(
    //             context
    //                 .config
    //                 .auth
    //                 .ok_or(CliError::UnAuthenticated)?
    //                 .id_token,
    //         )
    //         .with_sub(context.state.sub)
    //         .with_api_url(context.config.core.wukong_api_url)
    //         .build()?;
    //
    //     // let k8s_pods = client
    //     //     .fetch_kubernetes_pods(namespace)
    //     //     .await?
    //     //     .data
    //     //     .unwrap()
    //     //     .kubernetes_pods;
    //
    Ok(true)
}
//
// async fn has_permission() -> bool {
//     sleep(std::time::Duration::from_secs(2)).await;
//     true
// }
