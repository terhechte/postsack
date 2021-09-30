use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_path: PathBuf,
    pub emails_folder_path: PathBuf,
}

impl Config {
    pub fn new<A: AsRef<Path>>(db: A, mails: A) -> Self {
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
        }
    }
}
