use crate::database::{DBMessage, Database};
use crate::filesystem::RawEmailEntry;
use crate::types::{Config, EmailEntry};

use chrono::prelude::*;
use eyre::{bail, eyre, Result};
use rayon::prelude::*;
use std::thread::JoinHandle;

use std::{
    convert::{TryFrom, TryInto},
    path::Path,
};

pub enum ParseMessage {
    Total(usize),
    ParsedOne,
    Done,
}

pub type ProcessReceiver = crossbeam_channel::Receiver<Result<ParseMessage>>;

pub fn parse_emails(
    config: &Config,
    emails: Vec<RawEmailEntry>,
) -> Result<(ProcessReceiver, JoinHandle<Result<usize>>)> {
    // This channel is used to communicate the parsing progress.
    let (tx, rx) = crossbeam_channel::bounded(100);

    let config = config.clone();

    // Spawn all work into a new thread so it doesn't block the main thread.
    let handle = std::thread::spawn(move || {
        let total = emails.len();
        tracing::info!("Loaded {} emails", &total);

        // First, communicate the total amount of mails received
        if let Err(e) = tx.send(Ok(ParseMessage::Total(total))) {
            bail!("Channel Failure {:?}", &e);
        }

        // Create a new database connection
        let database = Database::new(config.database_path)?;

        // Consume the connection to begin the import. It will return the `handle` to use for
        // waiting for the database to finish importing, and the `sender` to submit work.
        let (sender, handle) = database.import();

        // Iterate over the mails..
        emails
            // in paralell..
            .par_iter()
            // parsing them
            .map(|raw_mail| (raw_mail.path(), parse_email(&raw_mail)))
            // and inserting them into SQLite
            .for_each(|(path, entry)| {
                if let Err(e) = tx.send(Ok(ParseMessage::ParsedOne)) {
                    tracing::error!("Channel Failure: {:?}", &e);
                }
                if let Err(e) = match entry {
                    Ok(mail) => sender.send(DBMessage::Mail(mail)),
                    Err(e) => sender.send(DBMessage::Error(e, path.to_path_buf())),
                } {
                    tracing::error!("Error Inserting into Database: {:?}", &e);
                }
            });

        // Tell SQLite there's no more work coming. This will exit the listening loop
        if let Err(e) = sender.send(DBMessage::Done) {
            bail!("Channel Failure {:?}", &e);
        }

        // Wait for SQLite to finish parsing
        tracing::info!("Waiting for SQLite to finish");
        let output = match handle.join() {
            Ok(Ok(count)) => Ok(count),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(eyre::eyre!("Join Error: {:?}", &e)),
        };

        // Tell the caller that we're done processing. This will allow leaving the
        // display loop
        if let Err(e) = tx.send(Ok(ParseMessage::Done)) {
            bail!("Channel Failure {:?}", &e);
        }

        output
    });
    Ok((rx, handle))
}

fn parse_email(raw_entry: &RawEmailEntry) -> Result<EmailEntry> {
    let content = raw_entry.read()?;
    parse_email_parser(&raw_entry, &content)
}

fn parse_email_parser(raw_entry: &RawEmailEntry, content: &Vec<u8>) -> Result<EmailEntry> {
    match email_parser::email::Email::parse(&content) {
        Ok(email) => (raw_entry.path(), email).try_into(),
        Err(error) => {
            //let content_string = String::from_utf8(content.clone())?;
            //println!("{}|{}", &error, &raw_entry.eml_path.display());
            Err(eyre!(
                "Could not parse email: {:?} [{}]",
                &error,
                raw_entry.path().display()
            ))
        }
    }
}

impl<'a> TryFrom<(&Path, email_parser::email::Email<'a>)> for EmailEntry {
    type Error = eyre::Report;
    fn try_from(content: (&Path, email_parser::email::Email)) -> Result<Self, Self::Error> {
        let (path, email) = content;
        let domain = email.sender.address.domain.to_string();
        let local_part = email.sender.address.local_part.to_string();
        let datetime = emaildatetime_to_chrono(&email.date);
        let subject = email.subject.map(|e| e.to_string()).unwrap_or_default();

        Ok(EmailEntry {
            path: path.to_path_buf(),
            domain,
            local_part,
            datetime,
            subject,
        })
    }
}

fn emaildatetime_to_chrono(dt: &email_parser::time::DateTime) -> chrono::DateTime<Utc> {
    Utc.ymd(
        dt.date.year as i32,
        dt.date.month_number() as u32,
        dt.date.day as u32,
    )
    .and_hms(
        dt.time.time.hour as u32,
        dt.time.time.minute as u32,
        dt.time.time.second as u32,
    )
}
