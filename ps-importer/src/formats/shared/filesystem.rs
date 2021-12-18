use ps_core::eyre::{bail, Result};
use ps_core::tracing::{self, trace};
use rayon::prelude::*;

use std::path::{Path, PathBuf};

use ps_core::{Message, MessageSender};

/// Call `FolderAction` on all files in all sub folders in
/// folder `folder`.
pub fn folders_in<FolderAction, ActionResult, P>(
    folder: P,
    sender: MessageSender,
    action: FolderAction,
) -> Result<Vec<ActionResult>>
where
    P: AsRef<Path>,
    FolderAction: Fn(PathBuf, MessageSender) -> Result<Vec<ActionResult>> + Send + Sync,
    ActionResult: Send,
{
    let folder = folder.as_ref();
    if !folder.exists() {
        bail!("Folder {} does not exist", &folder.display());
    }
    // For progress reporting, we collect the iterator in order to
    // know how many items there are.
    let items: Vec<_> = std::fs::read_dir(&folder)?.collect();
    let total = items.len();
    sender.send(Message::ReadTotal(total))?;
    Ok(items
        .into_iter()
        .par_bridge()
        .filter_map(|entry| {
            let path = entry
                .map_err(|e| tracing::error!("{} {:?}", &folder.display(), &e))
                .ok()?
                .path();
            if !path.is_dir() {
                return None;
            }
            let sender = sender.clone();
            trace!("Reading folder {}", path.display());
            action(path.clone(), sender)
                .map_err(|e| tracing::error!("{} {:?}", path.display(), &e))
                .ok()
        })
        .flatten()
        .collect())
}

pub fn emails_in<O, F, P: AsRef<Path>>(path: P, sender: MessageSender, make: F) -> Result<Vec<O>>
where
    F: Fn(PathBuf) -> Option<O>,
    F: Send + Sync + 'static,
    O: Send + Sync,
{
    let path = path.as_ref();
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
            make(path)
        })
        .collect());
    // We're done reading the folder
    sender.send(Message::ReadOne).unwrap();
    result
}
