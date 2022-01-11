use ps_core::eyre::eyre;
use ps_core::tracing;
use rayon::prelude::*;
use walkdir::WalkDir;

use super::{Config, ImporterFormat, Message, MessageSender, Result};

use super::shared::parse::ParseableEmail;
use ps_core::EmailMeta;

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
fn inner_emails(config: &Config, sender: MessageSender) -> Result<Vec<Mail>> {
    // find all files ending in .mbox
    let mboxes: Vec<PathBuf> = WalkDir::new(&config.emails_folder_path)
        .into_iter()
        .filter_map(|e| match e {
            Ok(n)
                if n.path().is_file()
                    && n.path()
                        .to_str()
                        .map(|e| e.contains(".mbox"))
                        .unwrap_or(false) =>
            {
                tracing::trace!("Found mbox file {}", n.path().display());
                Some(n.path().to_path_buf())
            }
            Err(e) => {
                tracing::info!("Could not read folder: {}", e);
                if let Err(e) = sender.send(Message::Error(eyre!("Could not read folder: {:?}", e)))
                {
                    tracing::error!("Error sending error {}", e);
                }
                None
            }
            _ => None,
        })
        .collect();

    let mails: Vec<Mail> = mboxes
        .into_par_iter()
        .filter_map(|mbox_file| {
            let mbox = match mbox_reader::MboxFile::from_file(&mbox_file) {
                Ok(n) => n,
                Err(e) => {
                    tracing::error!(
                        "Could not open mbox file at {}: {}",
                        &mbox_file.display(),
                        e
                    );
                    return None;
                }
            };
            let inner_mails: Vec<Mail> = mbox
                .iter()
                .filter_map(|e| {
                    let content = match e.message() {
                        Some(n) => n,
                        None => {
                            tracing::error!("Could not parse mail at offset {}", e.offset());
                            return None;
                        }
                    };
                    sender.send(Message::ReadOne).ok();
                    Some(Mail {
                        path: mbox_file.clone(),
                        content: content.to_owned(),
                    })
                })
                .collect();
            Some(inner_mails)
        })
        .flatten()
        .collect();
    Ok(mails)
}

impl ImporterFormat for Mbox {
    type Item = Mail;

    fn default_path() -> Option<std::path::PathBuf> {
        None
    }

    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>> {
        inner_emails(config, sender)
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
        // The filename is a tag, e.g. `INBOX.mbox`, `WORK.mbox`
        if let Some(prefix) = self.path.file_stem() {
            if let Some(s) = prefix.to_str().map(|s| s.to_owned()) {
                return Ok(Some(EmailMeta {
                    tags: vec![s],
                    is_seen: false,
                }));
            }
        }
        Ok(None)
    }
}
