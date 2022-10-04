pub mod error;
pub mod table;

#[derive(Default)]
pub enum OutputFormat {
    #[default]
    Display,
    Json,
}


