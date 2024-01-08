use crate::error::WKCliError;

pub async fn handle_application_init() -> Result<bool, WKCliError> {
    println!("This is the application init command");
    Ok(true)
}
