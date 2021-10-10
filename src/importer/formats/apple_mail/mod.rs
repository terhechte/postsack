mod filesystem;
mod mail;

use super::{Config, ImporterFormat, MessageSender, Result};

#[derive(Default)]
pub struct AppleMail {}

impl ImporterFormat for AppleMail {
    type Item = mail::Mail;
    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>> {
        filesystem::read_emails(config, sender)
    }
}
