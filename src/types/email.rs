use chrono::prelude::*;
use std::path::PathBuf;

/// Representation of an email
#[derive(Debug)]
pub struct EmailEntry {
    pub path: PathBuf,
    pub domain: String,
    pub local_part: String,
    pub datetime: chrono::DateTime<Utc>,
    pub subject: String,
}
