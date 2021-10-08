mod filesystem;
mod meta;
mod raw_email;

use super::{Config, ImporterFormat, MessageSender, Result};

#[derive(Default)]
pub struct Gmail {}

impl ImporterFormat for Gmail {
    type Item = raw_email::RawEmailEntry;
    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>> {
        filesystem::read_emails(config, sender)
    }
}
