#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

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

    let collector = tracing_subscriber::registry().with(fmt::layer().with_writer(std::io::stdout));

    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
}

/// Create a config for the `cli` and validate the input
pub fn make_config() -> types::Config {
    use std::path::Path;
    use types::FormatType;
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
    let format: FormatType = arguments
        .get(4)
        .unwrap_or_else(|| usage("Missing sender email address argument"))
        .into();

    let database_path = Path::new(database);
    if database_path.is_dir() {
        panic!(
            "Database Path can't be a directory: {}",
            &database_path.display()
        );
    }
    let emails_folder_path = Path::new(folder);
    // For non-mbox files, we make sure we have a directory
    if format != FormatType::Mbox && !emails_folder_path.is_dir() {
        panic!(
            "Emails Folder Path is not a directory: {}",
            &emails_folder_path.display()
        );
    }

    match crate::types::Config::new(Some(database), folder, vec![sender.to_string()], format) {
        Ok(n) => n,
        Err(r) => panic!("Error: {:?}", &r),
    }
}

fn usage(error: &'static str) -> ! {
    println!("Usage: cli [email-folder] [database-path] [sender-email-address] [format]");
    println!("\tExample: cli ~/Library/Mails/V9/ ./db.sqlite my-address@gmail.com apple");
    panic!("{}", error);
}
