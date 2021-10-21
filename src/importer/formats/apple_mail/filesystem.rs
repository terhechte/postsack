//! We use a stubbornly stupid algorithm where we just
//! recursively drill down into the appropriate folder
//! until we find `emlx` files and return those.

use eyre::Result;
use rayon::prelude::*;
use walkdir::WalkDir;

use super::super::shared::filesystem::emails_in;
use super::super::{Message, MessageSender};
use crate::types::Config;

use super::mail::Mail;
use std::path::PathBuf;

pub fn read_emails(config: &Config, sender: MessageSender) -> Result<Vec<Mail>> {
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
                Some(n.path().to_path_buf())
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
                    None
                }
            },
        )
        .flatten()
        .collect();
    Ok(mails)
}
