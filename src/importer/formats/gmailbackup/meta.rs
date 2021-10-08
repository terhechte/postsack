use chrono::prelude::*;

use eyre::{bail, Result};
use serde::Deserialize;
use serde_json;

use super::super::shared::email::EmailMeta;
use super::raw_email::RawEmailEntry;

#[derive(Deserialize, Debug, Clone)]
pub struct Meta {
    pub msg_id: String,
    pub subject: String,
    pub labels: Vec<String>,
    pub flags: Vec<String>,
    internal_date: i64,

    #[serde(skip, default = "Utc::now")]
    pub created: DateTime<Utc>,
}

impl Meta {
    pub fn is_seen(&self) -> bool {
        self.labels.contains(&"\\seen".to_owned())
    }
}

impl From<Meta> for EmailMeta {
    fn from(meta: Meta) -> Self {
        let is_seen = meta.is_seen();
        EmailMeta {
            tags: meta.labels,
            is_seen,
        }
    }
}

pub fn parse_meta(raw_entry: &RawEmailEntry) -> Result<Meta> {
    let content = match raw_entry.read_gmail_meta() {
        None => bail!("No Gmail Meta Information Available"),
        Some(content) => content?,
    };
    let mut meta: Meta = serde_json::from_slice(&content)?;
    meta.created = Utc.timestamp(meta.internal_date, 0);
    Ok(meta)
}
