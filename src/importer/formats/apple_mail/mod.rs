mod filesystem;
mod mail;

use super::{Config, ImporterFormat, MessageSender, Result};

#[derive(Default)]
pub struct AppleMail {}

impl ImporterFormat for AppleMail {
    type Item = mail::Mail;

    fn default_path() -> Option<&'static std::path::Path> {
        Some(std::path::Path::new("~/Library/Mail"))
    }

    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>> {
        filesystem::read_emails(config, sender)
    }
}
