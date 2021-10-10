use eyre::Result;

use super::super::shared::filesystem::{emails_in, folders_in};
use super::super::MessageSender;
use super::raw_email::RawEmailEntry;
use crate::types::Config;

pub fn read_emails(config: &Config, sender: MessageSender) -> Result<Vec<RawEmailEntry>> {
    Ok(folders_in(
        &config.emails_folder_path,
        sender,
        |path, sender| emails_in(path, sender, RawEmailEntry::new),
    )?)
}
