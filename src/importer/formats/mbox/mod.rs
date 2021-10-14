//! FIXME: Implement our own Mailbox reader. This one is terrible!
//! use jetsci for efficient searching:
//! https://github.com/shepmaster/jetscii
//! (or aho corasick)
//! Here's the ref: file:///Users/terhechte/Development/Rust/gmaildb/target/doc/src/mbox_reader/lib.rs.html#65-67
//! Make it so that I can hold the mbox in the struct below

use eyre::bail;
use mbox_reader;
use tracing;

use super::{Config, ImporterFormat, MessageSender, Result};

use super::shared::email::EmailMeta;
use super::shared::parse::ParseableEmail;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub struct Mail {
    path: PathBuf,
    /// For now, we go with a very simple implementation:
    /// Each mal will have a heap-allocated vec of the corresponding
    /// bytes in the mbox.
    /// This wastes a lot of allocations and shows the limits of our current abstraction.
    /// It would be better to just save the headers and ignore the rest.
    content: Vec<u8>,
}

#[derive(Default)]
pub struct Mbox;

/// The inner parsing code
fn inner_emails(config: &Config) -> Result<Vec<Mail>> {
    if config
        .emails_folder_path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        != Some("mbox")
    {
        bail!("Path does not point to an .mbox file")
    }

    let mbox = mbox_reader::MboxFile::from_file(config.emails_folder_path.as_path())?;

    let path = config.emails_folder_path.clone();
    Ok(mbox
        .iter()
        .filter_map(|e| {
            let content = match e.message() {
                Some(n) => n,
                None => {
                    tracing::error!("Could not parse mail at offset {}", e.offset());
                    return None;
                }
            };
            Some(Mail {
                path: path.clone(),
                content: content.to_owned(),
            })
        })
        .collect())
}

impl ImporterFormat for Mbox {
    type Item = Mail;
    fn emails(&self, config: &Config, _sender: MessageSender) -> Result<Vec<Self::Item>> {
        inner_emails(config)
    }
}

impl ParseableEmail for Mail {
    fn prepare(&mut self) -> Result<()> {
        Ok(())
    }
    fn message(&self) -> Result<Cow<'_, [u8]>> {
        Ok(self.content.as_slice().into())
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn meta(&self) -> Result<Option<EmailMeta>> {
        Ok(None)
    }
}
