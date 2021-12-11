//! We use a stubbornly stupid algorithm where we just
//! recursively drill down into the appropriate folder
//! until we find `emlx` files and return those.

use eyre::{eyre, Result};
use rayon::prelude::*;
use walkdir::WalkDir;

use super::super::shared::filesystem::emails_in;
use super::super::{Message, MessageSender};
use crate::types::Config;

use super::mail::Mail;
use std::path::PathBuf;

pub fn read_emails(config: &Config, sender: MessageSender) -> Result<Vec<Mail>> {
    // on macOS, we might need permission for the `Library` folder...
    match std::fs::read_dir(&config.emails_folder_path) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            #[cfg(target_os = "macos")]
            std::io::ErrorKind::PermissionDenied => {
                tracing::info!("Could not read folder: {}", e);
                if let Err(e) = sender.send(Message::MissingPermissions) {
                    tracing::error!("Error sending: {}", e);
                }
                // We should return early now, otherwise the code below will send a different
                // error
                return Ok(Vec::new());
            }
            _ => {
                if let Err(e) = sender.send(Message::Error(eyre!("Error: {:?}", &e))) {
                    tracing::error!("Error sending: {}", e);
                }
            }
        },
    }

    // As `walkdir` does not support `par_iter` (see https://www.reddit.com/r/rust/comments/6eif7r/walkdir_users_we_need_you/)
    // - -we first collect all folders,
    // then all sub-folders in those ending in mboxending in .mbox and then iterate over them in paralell
    let folders: Vec<PathBuf> = WalkDir::new(&config.emails_folder_path)
        .into_iter()
        .filter_map(|e| match e {
            Ok(n)
                if n.path().is_dir()
                    && n.path()
                        .to_str()
                        .map(|e| e.contains(".mbox"))
                        .unwrap_or(false) =>
            {
                tracing::trace!("Found folder {}", n.path().display());
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
    sender.send(Message::ReadTotal(folders.len()))?;
    let mails: Vec<Mail> = folders
        .into_par_iter()
        .filter_map(
            |path| match emails_in(path.clone(), sender.clone(), Mail::new) {
                Ok(n) => Some(n),
                Err(e) => {
                    tracing::error!("{} {:?}", path.display(), &e);
                    if let Err(e) = sender.send(Message::Error(eyre!(
                        "Could read mails in {}: {:?}",
                        path.display(),
                        e
                    ))) {
                        tracing::error!("Error sending error {}", e);
                    }
                    None
                }
            },
        )
        .flatten()
        .collect();
    Ok(mails)
}
