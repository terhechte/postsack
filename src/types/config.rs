use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum ImporterFormat {
    AppleMail,
    GmailVault,
}

impl From<&String> for ImporterFormat {
    fn from(format: &String) -> Self {
        match format.as_str() {
            "apple" => ImporterFormat::AppleMail,
            "gmailvault" => ImporterFormat::GmailVault,
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
    pub format: ImporterFormat,
}

impl Config {
    pub fn new<A: AsRef<Path>>(
        db: A,
        mails: A,
        sender_email: String,
        format: ImporterFormat,
    ) -> Self {
        let database_path = db.as_ref().to_path_buf();
        if database_path.is_dir() {
            panic!(
                "Database Path can't be a directory: {}",
                &database_path.display()
            );
        }
        let emails_folder_path = mails.as_ref().to_path_buf();
        if !emails_folder_path.is_dir() {
            panic!(
                "Emails Folder Path is not a directory: {}",
                &emails_folder_path.display()
            );
        }
        Config {
            database_path,
            emails_folder_path,
            sender_email,
            format,
        }
    }
}
