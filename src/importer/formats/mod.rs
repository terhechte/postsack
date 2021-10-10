pub use eyre::Result;

mod apple_mail;
mod gmailbackup;
pub mod shared;

pub use apple_mail::AppleMail;
pub use gmailbackup::Gmail;

pub use crate::types::Config;
use shared::parse::ParseableEmail;

pub use super::{Message, MessageReceiver, MessageSender};

/// This is implemented by the various formats
/// to define how they return email data.
pub trait ImporterFormat: Send + Sync {
    type Item: ParseableEmail;

    /// Return all the emails in this format.
    /// Use the sneder to give progress updates via the `ReadProgress` case.
    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>>;
}
