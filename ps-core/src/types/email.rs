use chrono::prelude::*;
use std::path::PathBuf;

pub type Tag = String;

/// This is based on additional information in some systems such as
/// Gmail labels or Apple Mail tags or Apple XML
#[derive(Debug, Default)]
pub struct EmailMeta {
    pub tags: Vec<Tag>,
    pub is_seen: bool,
}

const TAG_SEP: &str = ":|:";

impl EmailMeta {
    pub fn tags_from_string(tag_string: &str) -> Vec<String> {
        tag_string.split(TAG_SEP).map(|e| e.to_string()).collect()
    }

    pub fn from(is_seen: bool, tag_string: &str) -> Self {
        let tags = EmailMeta::tags_from_string(tag_string);
        EmailMeta { tags, is_seen }
    }

    pub fn tags_string(&self) -> String {
        self.tags.join(TAG_SEP)
    }
}

/// Representation of an email
#[derive(Debug)]
pub struct EmailEntry {
    pub path: PathBuf,
    pub sender_domain: String,
    pub sender_local_part: String,
    pub sender_name: String,
    pub datetime: chrono::DateTime<Utc>,
    pub subject: String,
    /// The amount of `to:` adresses
    pub to_count: usize,
    /// When this email was send to a group, the group name
    pub to_group: Option<String>,
    /// The first address and name in `To`, if any
    pub to_first: Option<(String, String)>,
    pub is_reply: bool,
    /// Was this email send from the account we're importing?
    pub is_send: bool,
    pub meta: Option<EmailMeta>,
}
