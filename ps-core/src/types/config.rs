use eyre::{eyre, Result};
use serde_json::Value;
use strum::{self, IntoEnumIterator};
use strum_macros::{EnumIter, IntoStaticStr};

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::str::FromStr;

// FIXME: This abstraction should be in the `ps-importer` crate with only
// a protocol here.

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, EnumIter)]
pub enum FormatType {
    AppleMail,
    GmailVault,
    Mbox,
    #[cfg(not(target_os = "windows"))]
    Maildir,
}

impl FormatType {
    pub fn all_cases() -> impl Iterator<Item = FormatType> {
        FormatType::iter()
    }

    pub fn name(&self) -> &'static str {
        match self {
            FormatType::AppleMail => "Apple Mail",
            FormatType::GmailVault => "Gmail Vault Download",
            FormatType::Mbox => "Mbox",
            #[cfg(not(target_os = "windows"))]
            FormatType::Maildir => "Maildir",
        }
    }
}

impl Default for FormatType {
    /// We return a different default, based on the platform we're on
    /// FIXME: We don't have support for Outlook yet, so on windows we go with Mbox as well
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        return FormatType::AppleMail;

        #[cfg(target_os = "windows")]
        return FormatType::Mbox;

        #[cfg(not(target_os = "macos"))]
        return FormatType::Maildir;
    }
}

impl From<&String> for FormatType {
    fn from(format: &String) -> Self {
        FormatType::from(format.as_str())
    }
}

impl From<&str> for FormatType {
    fn from(format: &str) -> Self {
        match format {
            "apple" => FormatType::AppleMail,
            "gmailvault" => FormatType::GmailVault,
            "mbox" => FormatType::Mbox,
            #[cfg(not(target_os = "windows"))]
            "maildir" => FormatType::Maildir,
            _ => panic!("Unknown format: {}", &format),
        }
    }
}

impl From<FormatType> for String {
    fn from(format: FormatType) -> Self {
        match format {
            FormatType::AppleMail => "apple".to_owned(),
            FormatType::GmailVault => "gmailvault".to_owned(),
            FormatType::Mbox => "mbox".to_owned(),
            #[cfg(not(target_os = "windows"))]
            FormatType::Maildir => "maildir".to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    /// The path to where the database should be stored
    pub database_path: PathBuf,
    /// The path where the emails are
    pub emails_folder_path: PathBuf,
    /// The addresses used to send emails
    pub sender_emails: HashSet<String>,
    /// The importer format we're using
    pub format: FormatType,
    /// Did the user intend to keep the database
    /// (e.g. is the database path temporary?)
    pub persistent: bool,
}

impl Config {
    /// Construct a config from a hashmap of field values.
    /// For missing fields, take a reasonable default value,
    /// in order to be somewhat backwards compatible.
    pub fn from_fields<P: AsRef<Path>>(path: P, fields: HashMap<String, Value>) -> Result<Config> {
        // The following fields are of version 1.0, so they should aways exist
        let emails_folder_path_str = fields
            .get("emails_folder_path")
            .ok_or_else(|| eyre!("Missing config field emails_folder_path"))?
            .as_str()
            .ok_or_else(|| eyre!("Invalid field type for emails_folder_path"))?;
        let emails_folder_path = PathBuf::from_str(emails_folder_path_str).map_err(|e| {
            eyre!(
                "Invalid emails_folder_path: {}: {}",
                &emails_folder_path_str,
                e
            )
        })?;
        #[allow(clippy::needless_collect)]
        let sender_emails: Vec<String> = fields
            .get("sender_emails")
            .map(|v| v.as_str().map(|e| e.to_string()))
            .flatten()
            .ok_or_else(|| eyre!("Missing config field sender_emails"))?
            .split(',')
            .map(|e| e.trim().to_owned())
            .collect();
        let format = fields
            .get("format")
            .map(|e| e.as_str())
            .flatten()
            .map(FormatType::from)
            .ok_or_else(|| eyre!("Missing config field format_type"))?;
        let persistent = fields
            .get("persistent")
            .map(|e| e.as_bool())
            .flatten()
            .ok_or_else(|| eyre!("Missing config field persistent"))?;
        Ok(Config {
            database_path: path.as_ref().to_path_buf(),
            emails_folder_path,
            sender_emails: HashSet::from_iter(sender_emails.into_iter()),
            format,
            persistent,
        })
    }

    pub fn new<A: AsRef<Path>>(
        db: Option<A>,
        mails: A,
        sender_emails: Vec<String>,
        format: FormatType,
    ) -> eyre::Result<Self> {
        // If we don't have a database path, we use a temporary folder.
        let persistent = db.is_some();

        #[cfg(target_arch = "wasm32")]
        let database_path = PathBuf::new();

        #[cfg(not(target_arch = "wasm32"))]
        let database_path = match db {
            Some(n) => n.as_ref().to_path_buf(),
            None => {
                let number = random_filename();
                let folder = "postsack";
                let filename = format!("{}.sqlite", number);
                let mut temp_dir = std::env::temp_dir();
                temp_dir.push(folder);
                // the folder has to be created
                std::fs::create_dir_all(&temp_dir)?;
                temp_dir.push(filename);
                temp_dir
            }
        };
        Ok(Config {
            database_path,
            emails_folder_path: mails.as_ref().to_path_buf(),
            sender_emails: HashSet::from_iter(sender_emails.into_iter()),
            format,
            persistent,
        })
    }

    pub fn into_fields(&self) -> Option<HashMap<String, Value>> {
        let mut new = HashMap::new();
        new.insert(
            "database_path".to_owned(),
            self.database_path.to_str()?.into(),
        );
        new.insert(
            "emails_folder_path".to_owned(),
            self.emails_folder_path.to_str()?.into(),
        );
        new.insert("persistent".to_owned(), self.persistent.into());
        new.insert(
            "sender_emails".to_owned(),
            self.sender_emails
                .iter()
                .cloned()
                .collect::<Vec<String>>()
                .join(",")
                .into(),
        );
        let format: String = self.format.into();
        new.insert("format".to_owned(), format.into());

        Some(new)
    }
}

fn random_filename() -> String {
    use rand::Rng;
    let number: u32 = rand::thread_rng().gen();
    let filename = format!("{}.sqlite", number);
    filename
}
