use tracing_subscriber::EnvFilter;

pub mod database;
#[cfg(feature = "gui")]
pub mod gui;
pub mod importer;
mod model;
pub mod types;

pub fn setup_tracing() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

pub fn make_config() -> types::Config {
    let arguments: Vec<String> = std::env::args().collect();
    let folder = arguments
        .get(1)
        .unwrap_or_else(|| panic!("Missing folder path argument"));
    let database = arguments
        .get(2)
        .unwrap_or_else(|| panic!("Missing database path argument"));
    let config = crate::types::Config::new(database, folder);
    config
}
