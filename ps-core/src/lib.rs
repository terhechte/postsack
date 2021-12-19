//! # Core types, traits and imports
//!
//! This crate is responsible for the core query constructs and core types.
//! It also re-exports the crates that need to be used in the rest of the workspace.
//!
//! It contains the following modules:
//!
//! ## database
//!
//! Query and Query result abstractions. These are the types that are used by the
//! gui to request data from the database and from the database to send the data
//! back to the gui.
//! Also, the required traits to implement a generic database type for the importer
//! and the gui.
//!
//! ## importer
//!
//! Types and traits that define how a data importer works. Types conforming to these
//! traits are used in the gui to import data into a database.
//!
//! ## model
//!
//! All the functionality related to the view related model requirements. Query data,
//! generate 2d segmentation rectangles out of the data, set filters, select segments,
//! basically all the processing of the data.
//!
//! ## message_adapter
//!
//! A abstraction on top of any `importer` to simplify using them.
//!
//! ## types
//!
//! Multiple types which are needed across the codebase, such as the `Configuration` or
//! the representation of an email.
//!
//! # Usage
//!
//! The core library itself needs a database and an importer to be useful. Once these
//! types exist, core will use the importer to fill the database and then the types
//! in `model` (e.g. `engine.rs`) can be used to perform segmentations of the data.
//!
//! Currently the model / engine is implemented in a non-intuitive async (not in the
//! Rust async way) way but this is due to finding a solution that would work with
//! egui. See: <https://github.com/terhechte/postsack/issues/11>

mod database;
mod importer;
pub mod message_adapter;
pub mod model;
mod types;

pub use database::database_like::{DatabaseLike, DatabaseQuery};
pub use database::db_message::DBMessage;
pub use database::query::{Field, Filter, OtherQuery, Query, ValueField, AMOUNT_FIELD_NAME};
pub use database::query_result::{QueryResult, QueryRow};
pub use importer::{Importerlike, Message, MessageReceiver, MessageSender};
pub use types::{Config, EmailEntry, EmailMeta, FormatType};

// Re-Export some dependencies so they don't
// need to be listed again in other Cargo tomls
pub use chrono;
pub use crossbeam_channel;
pub use eyre;
pub use rand;
pub use serde_json::{self, Value};
pub use tracing;

// Tracing

use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

pub fn setup_tracing() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error")
    }

    let collector = tracing_subscriber::registry().with(fmt::layer().with_writer(std::io::stdout));

    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
}
