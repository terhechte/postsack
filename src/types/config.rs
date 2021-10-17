use strum::{self, IntoEnumIterator};
use strum_macros::{EnumIter, IntoStaticStr};

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
    pub fn default_path(&self) -> Option<&'static Path> {
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
    /// The address used to send emails
    pub sender_email: String,
    /// The importer format we're using
    pub format: FormatType,
}

impl Config {
    pub fn new<A: AsRef<Path>>(db: A, mails: A, sender_email: String, format: FormatType) -> Self {
        Config {
            database_path: db.as_ref().to_path_buf(),
            emails_folder_path: mails.as_ref().to_path_buf(),
            sender_email,
            format,
        }
    }
}
