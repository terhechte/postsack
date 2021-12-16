use std::path::PathBuf;

pub use eyre::Result;

mod apple_mail;
mod gmailbackup;
mod mbox;
pub mod shared;

pub use apple_mail::AppleMail;
pub use gmailbackup::Gmail;
pub use mbox::Mbox;

use shared::parse::ParseableEmail;

pub use ps_core::{Config, Message, MessageReceiver, MessageSender};

/// This is implemented by the various formats
/// to define how they return email data.
pub trait ImporterFormat: Send + Sync {
    type Item: ParseableEmail;

    /// The default location path where the data for this format resides
    /// on system. If there is none (such as for mbox) return `None`
    fn default_path() -> Option<PathBuf>;

    /// Return all the emails in this format.
    /// Use the sneder to give progress updates via the `ReadProgress` case.
    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>>;
}
