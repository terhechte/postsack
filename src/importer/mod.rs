use crossbeam_channel;

mod formats;
mod message_adapter;

pub use formats::shared::email::{EmailEntry, EmailMeta};

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

// pub enum Message {}

// pub struct Importer<FORMAT: ImporterFormat> {
//     format: FORMAT,
//     sender: Sender<Message>,
// }
