//! We use a stubbornly stupid algorithm where we just
//! recursively drill down into the appropriate folder
//! until we find `emlx` files and return those.

use eyre::Result;
use rayon::prelude::*;
use tracing::trace;
use walkdir::WalkDir;

use super::super::shared::filesystem::folders_in;
use super::super::{Message, MessageSender};
use super::raw_email::RawEmailEntry;
use crate::types::Config;

use std::path::Path;

fn test_walkdir() {
    for entry in WalkDir::new("foo").int_par_iter().filter_map(|e| e.ok()) {
        println!("{}", entry.path().display());
    }
}

pub fn read_emails(config: &Config, sender: MessageSender) -> Result<Vec<RawEmailEntry>> {
    Ok(folders_in(&config.emails_folder_path, sender, read_folder)?)
}

fn read_folder(path: &Path, sender: MessageSender) -> Result<Vec<RawEmailEntry>> {
    let result = Ok(std::fs::read_dir(path)?
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
            RawEmailEntry::new(path)
        })
        .collect());
    // We're done reading the folder
    sender.send(Message::ReadOne).unwrap();
    result
}
