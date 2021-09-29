use eyre::{bail, Result};
use rayon::prelude::*;
use std::io::prelude::*;
use std::{io, path::PathBuf};
use thiserror;
use tracing_subscriber::EnvFilter;

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::database::Database;

mod database;
mod emails;

#[derive(Debug, thiserror::Error)]
enum GmailDBError {
    #[error("Missing folder argument")]
    MissingFolder,
}

fn main() -> Result<()> {
    setup();
    let arguments: Vec<String> = std::env::args().collect();
    let folder = arguments.get(1).ok_or(GmailDBError::MissingFolder)?;
    process_folder(&folder)?;
    //process_email(&folder)?;
    Ok(())
}

fn process_email(path: &str) -> Result<()> {
    let entry = emails::RawEmailEntry::new(&path);
    let mail = emails::read_email(&entry).unwrap();
    Ok(())
}

fn process_folder(folder: &str) -> Result<()> {
    let emails = emails::Emails::new(&folder)?;
    let total = emails.len();

    println!("Done Loading {} emails", &total);

    let mut database = Database::new().expect("Expect a valid database");

    let sender = database.process();

    use database::DBMessage;
    emails
        .emails
        //.par_iter()
        .iter()
        .map(|raw_mail| (&raw_mail, emails::read_email(&raw_mail)))
        .for_each(|(raw_mail, entry)| {
            if let Err(e) = match entry {
                Ok(mail) => sender.send(DBMessage::Mail(mail)),
                Err(e) => sender.send(DBMessage::Error(e, raw_mail.path())),
            } {
                tracing::info!("Error Inserting into Database: {:?}", &e);
            }
        });

    sender.send(database::DBMessage::Done).unwrap();
    while !sender.is_empty() {}
    Ok(())
}

fn setup() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
