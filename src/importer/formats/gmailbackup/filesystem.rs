use eyre::Result;
use rayon::prelude::*;

use super::super::shared::filesystem::folders_in;
use super::raw_email::RawEmailEntry;
use crate::types::Config;

use std::path::Path;

pub fn read_emails(config: &Config) -> Result<Vec<RawEmailEntry>> {
    Ok(folders_in(&config.emails_folder_path, read_folder)?)
}

fn read_folder(path: &Path) -> Result<Vec<RawEmailEntry>> {
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
            RawEmailEntry::new(path)
        })
        //.take(50)
        .collect())
}
