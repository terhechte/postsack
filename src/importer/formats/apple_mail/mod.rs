mod mail;

/// FIXME: Not sure if the number changes with each macOS release?
const DEFAULT_FOLDER: &str = "~/Library/Mail/V8/";

use super::{Config, ImporterFormat, MessageSender, Result};

#[derive(Default)]
pub struct AppleMail {}

impl ImporterFormat for AppleMail {
    type Item = mail::Mail;
    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>> {
        panic!()
    }
}
