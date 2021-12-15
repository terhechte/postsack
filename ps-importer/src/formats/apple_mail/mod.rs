mod filesystem;
mod mail;

use shellexpand;
use std::{path::PathBuf, str::FromStr};

use super::{ImporterFormat, Result};
use ps_core::{Config, MessageSender};

#[derive(Default)]
pub struct AppleMail {}

impl ImporterFormat for AppleMail {
    type Item = mail::Mail;

    fn default_path() -> Option<PathBuf> {
        let path = shellexpand::tilde("~/Library/Mail");
        Some(PathBuf::from_str(&path.to_string()).unwrap())
    }

    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>> {
        filesystem::read_emails(config, sender)
    }
}
