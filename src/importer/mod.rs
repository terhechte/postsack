use crossbeam_channel;

mod formats;
mod importer;
mod message_adapter;

use crate::types::Config;
pub use formats::shared::email::{EmailEntry, EmailMeta};
pub use message_adapter::*;

use formats::ImporterFormat;

/// The message that informs of the importers progress
pub enum Message {
    /// How much progress are we making on reading the contents
    /// of the emails.
    /// The `usize` parameter marks the total amount of items to read - if it is known.
    /// The values here can vary wildly based on the type of Importer `Format` in use.
    /// A Gmail backup will list the folders and how many of them
    /// are already read. A mbox format will list other things as there
    /// no folders.
    ReadTotal(usize),
    /// Whenever an item out of the total is read, this message will be emitted
    ReadOne,
    /// Similar to [`ReadTotal`]
    WriteTotal(usize),
    /// Similar to `ReadOne`
    WriteOne,
    /// Once everything has been written, we need to wait for the database
    /// to sync
    FinishingUp,
    /// Finally, this indicates that we're done.
    Done,
}

pub type MessageSender = crossbeam_channel::Sender<Message>;
pub type MessageReceiver = crossbeam_channel::Receiver<Message>;

pub fn importer(config: &Config) -> Box<dyn importer::Importerlike> {
    match config.format {
        crate::types::ImporterFormat::AppleMail => Box::new(applemail_importer(config.clone())),
        crate::types::ImporterFormat::GmailVault => Box::new(gmail_importer(config.clone())),
    }
}

pub fn gmail_importer(config: Config) -> importer::Importer<formats::Gmail> {
    importer::Importer::new(config, formats::Gmail::default())
}

pub fn applemail_importer(config: Config) -> importer::Importer<formats::AppleMail> {
    importer::Importer::new(config, formats::AppleMail::default())
}
