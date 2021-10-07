use std::path::PathBuf;

use eyre::Report;

use crate::importer::types::EmailEntry;

/// Parameter for sending work to the database during `import`.
pub enum DBMessage {
    /// Send for a successfuly parsed mail
    Mail(EmailEntry),
    /// Send for any kind of error during reading / parsing
    Error(Report, PathBuf),
    /// Send once all parsing is done.
    /// This is used to break out of the receiving loop
    Done,
}
