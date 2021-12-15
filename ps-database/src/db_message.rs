use eyre::Report;

use ps_core::EmailEntry;

/// Parameter for sending work to the database during `import`.
pub enum DBMessage {
    /// Send for a successfuly parsed mail
    Mail(Box<EmailEntry>),
    /// Send for any kind of error during reading / parsing
    Error(Report),
    /// Send once all parsing is done.
    /// This is used to break out of the receiving loop
    Done,
}
