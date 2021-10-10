use tracing_subscriber::EnvFilter;

pub mod database;
#[cfg(feature = "gui")]
pub mod gui;
pub mod importer;
mod model;
pub mod types;

pub fn setup_tracing() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

pub fn make_config() -> types::Config {
    use types::ImporterFormat;
    let arguments: Vec<String> = std::env::args().collect();
    let folder = arguments
        .get(1)
        .unwrap_or_else(|| usage("Missing email folder argument"));
    let database = arguments
        .get(2)
        .unwrap_or_else(|| usage("Missing database path argument"));
    let sender = arguments
        .get(3)
        .unwrap_or_else(|| usage("Missing sender email address argument"));
    let format: ImporterFormat = arguments
        .get(4)
        .unwrap_or_else(|| usage("Missing sender email address argument"))
        .into();
    let config = crate::types::Config::new(database, folder, sender.to_string(), format);
    config
}

fn usage(error: &'static str) -> ! {
    println!("Usage: cli [email-folder] [database-path] [sender-email-address] [format]");
    println!("\tExample: cli ~/Library/Mails/V9/ ./db.sqlite my-address@gmail.com apple");
    panic!("{}", error);
}
