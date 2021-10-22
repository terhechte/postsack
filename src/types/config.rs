use rand::Rng;
use strum::{self, IntoEnumIterator};
use strum_macros::{EnumIter, IntoStaticStr};

use std::collections::HashSet;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, EnumIter)]
pub enum FormatType {
    AppleMail,
    GmailVault,
    Mbox,
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
        }
    }

    /// Forward the importer format location
    pub fn default_path(&self) -> Option<PathBuf> {
        use crate::importer::formats::{self, ImporterFormat};
        match self {
            FormatType::AppleMail => formats::AppleMail::default_path(),
            FormatType::GmailVault => formats::Gmail::default_path(),
            FormatType::Mbox => formats::Mbox::default_path(),
        }
    }
}

impl Default for FormatType {
    /// We return a different default, based on the platform we're on
    /// FIXME: We don't have support for Outlook yet, so on windows we go with Mbox as well
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        return FormatType::AppleMail;

        #[cfg(not(target_os = "macos"))]
        return FormatType::MboxVault;
    }
}

impl From<&String> for FormatType {
    fn from(format: &String) -> Self {
        match format.as_str() {
            "apple" => FormatType::AppleMail,
            "gmailvault" => FormatType::GmailVault,
            "mbox" => FormatType::Mbox,
            _ => panic!("Unknown format: {}", &format),
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
    pub fn new<A: AsRef<Path>>(
        db: Option<A>,
        mails: A,
        sender_emails: Vec<String>,
        format: FormatType,
    ) -> eyre::Result<Self> {
        // If we don't have a database path, we use a temporary folder.
        let persistent = db.is_some();
        let database_path = match db {
            Some(n) => n.as_ref().to_path_buf(),
            None => {
                let number: u32 = rand::thread_rng().gen();
                let folder = "gmaildb";
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
}
