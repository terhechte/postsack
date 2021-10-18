mod import;
mod startup;
mod visualize;

pub use import::Import;
pub use startup::Startup;
pub use visualize::{UIState, Visualize};

pub fn make_temporary_ui_config() -> crate::types::Config {
    crate::types::Config::new(
        "./db6.sql",
        "",
        "terhechte@me.com".to_string(),
        crate::types::FormatType::AppleMail,
    )
}
