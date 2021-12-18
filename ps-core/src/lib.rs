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
