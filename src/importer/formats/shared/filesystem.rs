use eyre::{bail, eyre, Result};
use rayon::prelude::*;

use std::io::Read;
use std::path::{Path, PathBuf};

pub fn folders_in<FolderAction, ActionResult, P>(
    folder: P,
    action: FolderAction,
) -> Result<Vec<ActionResult>>
where
    P: AsRef<Path>,
    FolderAction: Fn(&Path) -> Result<Vec<ActionResult>> + Send + Sync,
    ActionResult: Send,
{
    let folder = folder.as_ref();
    if !folder.exists() {
        bail!("Folder {} does not exist", &folder.display());
    }
    Ok(std::fs::read_dir(&folder)?
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
            action(&path)
                .map_err(|e| tracing::error!("{} {:?}", &path.display(), &e))
                .ok()
        })
        .flatten()
        .collect())
}
