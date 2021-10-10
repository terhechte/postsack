mod meta;
mod raw_email;

use super::shared::filesystem::{emails_in, folders_in};
use super::{Config, ImporterFormat, MessageSender, Result};
use raw_email::RawEmailEntry;

#[derive(Default)]
pub struct Gmail {}

impl ImporterFormat for Gmail {
    type Item = raw_email::RawEmailEntry;
    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>> {
        Ok(folders_in(
            &config.emails_folder_path,
            sender,
            |path, sender| emails_in(path, sender, RawEmailEntry::new),
        )?)
    }
}
