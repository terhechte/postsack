use eyre::Result;
use rayon::prelude::*;
use tracing::trace;

use super::super::shared::filesystem::folders_in;
use super::super::{Message, MessageSender};
use super::raw_email::RawEmailEntry;
use crate::types::Config;

use std::path::Path;

pub fn read_emails(config: &Config, sender: MessageSender) -> Result<Vec<RawEmailEntry>> {
    Ok(folders_in(&config.emails_folder_path, sender, read_folder)?)
}

fn read_folder(path: &Path, sender: MessageSender) -> Result<Vec<RawEmailEntry>> {
    Ok(std::fs::read_dir(path)?
        .into_iter()
        .par_bridge()
        .filter_map(|entry| {
            let path = entry
                .map_err(|e| tracing::error!("{} {:?}", &path.display(), &e))
                .ok()?
                .path();
            if path.is_dir() {
                return None;
            }
            trace!("Reading {}", &path.display());
            sender.send(Message::ReadOne).unwrap();
            RawEmailEntry::new(path)
        })
        //.take(50)
        .collect())
}
