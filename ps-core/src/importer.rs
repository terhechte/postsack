use crossbeam_channel;
use eyre::{Report, Result};
use std::thread::JoinHandle;

use crate::DatabaseLike;

pub trait Importerlike {
    fn import<Database: DatabaseLike + 'static>(
        self,
        database: Database,
    ) -> Result<(MessageReceiver, JoinHandle<Result<()>>)>;
}

/// The message that informs of the importers progress
#[derive(Debug)]
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
    /// An error happened during processing
    Error(eyre::Report),
    /// A special case for macOS, where a permission error means we have to grant this app
    /// the right to see the mail folder
    #[cfg(target_os = "macos")]
    MissingPermissions,
}

pub type MessageSender = crossbeam_channel::Sender<Message>;
pub type MessageReceiver = crossbeam_channel::Receiver<Message>;
