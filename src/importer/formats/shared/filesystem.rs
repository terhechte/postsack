use eyre::{bail, eyre, Result};
use rayon::prelude::*;
use tracing::trace;

use std::io::Read;
use std::path::{Path, PathBuf};

use super::super::{Message, MessageSender};

pub fn folders_in<FolderAction, ActionResult, P>(
    folder: P,
    sender: MessageSender,
    action: FolderAction,
) -> Result<Vec<ActionResult>>
where
    P: AsRef<Path>,
    FolderAction: Fn(&Path, MessageSender) -> Result<Vec<ActionResult>> + Send + Sync,
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
            action(&path, sender)
                .map_err(|e| tracing::error!("{} {:?}", &path.display(), &e))
                .ok()
        })
        .flatten()
        .collect())
}
