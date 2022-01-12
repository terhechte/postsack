use ps_core::eyre::eyre;
use ps_core::tracing;
use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};

use super::{Config, ImporterFormat, Message, MessageSender, Result};

use super::shared::parse::{MessageKind, ParseableEmail};
use maildir;
use ps_core::EmailMeta;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub struct Mail {
    path: PathBuf,
    data: Vec<u8>,
    is_seen: bool,
}

#[derive(Default)]
pub struct Maildir;

/// The folder finding code:
/// Find all folders that contain a `cur` or `new` folder
fn inner_folders(config: &Config, sender: MessageSender) -> Result<Vec<PathBuf>> {
    // Configure a walkdir entry to:
    // - go recursively
    // - follow directories starting with a `.`
    // - follow the root directory
    // - ignore everything else
    fn take_entry(entry: &DirEntry, root_path: &Path) -> bool {
        let is_dir = entry.path().is_dir();
        let contains_dot = entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false);
        if is_dir && contains_dot {
            return true;
        }
        return entry.path() == root_path;
    }

    let root_path = config.emails_folder_path.as_path();
    let iter = WalkDir::new(&config.emails_folder_path)
        .into_iter()
        .filter_entry(|e| take_entry(e, root_path));

    let folders: Vec<PathBuf> = iter
        .filter_map(|e| match e {
            Ok(n) if n.path().is_dir() => {
                // Check if this folder contains a cur or new
                let contents = std::fs::read_dir(n.path()).ok()?;
                for entry in contents {
                    if let Ok(dir_entry) = entry {
                        if let Some(Some(name)) = dir_entry.path().file_name().map(|e| e.to_str()) {
                            if name == "cur" || name == "new" {
                                tracing::trace!("Found folder {}", n.path().display());
                                return Some(n.path().to_path_buf());
                            }
                        }
                    }
                }
                None
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

    Ok(folders)
}

/// The inner email parsing code
fn inner_emails(path: &PathBuf, sender: MessageSender) -> Result<Vec<Mail>> {
    let maildir = maildir::Maildir::from(path.clone());
    let new_mails = maildir.list_new();
    let cur_mails = maildir.list_cur();

    tracing::info!("Finding maildirs in {}", path.display());

    let parsed_mails = new_mails
        .chain(cur_mails)
        .filter_map(|m| {
            let mail_entry = match m {
                Ok(n) => n,
                Err(e) => {
                    //tracing::error!("Could not parse mail: {}", e);
                    if let Err(e) = sender.send(Message::Error(eyre!("Could parse mail: {:?}", e)))
                    {
                        tracing::error!("Error sending error {}", e);
                    }
                    return None;
                }
            };
            Some((mail_entry.path().clone(), mail_entry.is_seen()))
        })
        .par_bridge()
        .filter_map(|(path, seen)| {
            let data = match std::fs::read(&path) {
                Ok(n) => n,
                Err(e) => {
                    tracing::error!("Could not read mail {}: {}", path.display(), e);
                    if let Err(e) = sender.send(Message::Error(eyre!(
                        "Could not read mail {}: {}",
                        path.display(),
                        e
                    ))) {
                        tracing::error!("Error sending error {}", e);
                    }
                    return None;
                }
            };
            Some(Mail {
                path: path.clone(),
                is_seen: seen,
                data,
            })
        })
        .collect();

    Ok(parsed_mails)
}

impl ImporterFormat for Maildir {
    type Item = Mail;

    fn default_path() -> Option<std::path::PathBuf> {
        None
    }

    fn emails(&self, config: &Config, sender: MessageSender) -> Result<Vec<Self::Item>> {
        // First get all the folders containing maildirs
        let folders = inner_folders(config, sender.clone())?;
        let mails = folders
            .par_iter()
            .filter_map(|folder| inner_emails(folder, sender.clone()).ok())
            .flatten()
            .collect();
        Ok(mails)
    }
}

impl ParseableEmail for Mail {
    fn prepare(&mut self) -> Result<()> {
        Ok(())
    }
    fn kind(&self) -> MessageKind<'_> {
        MessageKind::Data(Cow::Borrowed(self.data.as_slice()))
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn meta(&self) -> Result<Option<EmailMeta>> {
        Ok(Some(EmailMeta {
            tags: Vec::new(),
            is_seen: self.is_seen,
        }))
    }
}
