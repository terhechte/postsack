use super::super::filesystem::RawEmailEntry;
use super::super::types::EmailEntry;
use super::gmail_meta;
use crate::database::{DBMessage, Database};
use crate::types::Config;

use chrono::prelude::*;
use email_parser::address::{Address, EmailAddress, Mailbox};
use eyre::{bail, eyre, Result};
use rayon::prelude::*;
use std::thread::JoinHandle;

use std::convert::{TryFrom, TryInto};

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
        Ok(email) => (raw_entry, email).try_into(),
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

impl<'a> TryFrom<(&RawEmailEntry, email_parser::email::Email<'a>)> for EmailEntry {
    type Error = eyre::Report;
    fn try_from(
        content: (&RawEmailEntry, email_parser::email::Email),
    ) -> Result<Self, Self::Error> {
        let (entry, email) = content;
        let path = entry.path();
        let (sender_name, _, sender_local_part, sender_domain) = mailbox_to_string(&email.sender);

        let datetime = emaildatetime_to_chrono(&email.date);
        let subject = email.subject.map(|e| e.to_string()).unwrap_or_default();

        let to_count = match email.to.as_ref() {
            Some(n) => n.len(),
            None => 0,
        };
        let to = match email.to.as_ref().map(|v| v.first()).flatten() {
            Some(n) => address_to_name_string(n),
            None => None,
        };
        let to_group = to.as_ref().map(|e| e.0.clone()).flatten();
        let to_first = to.as_ref().map(|e| (e.1.clone(), e.2.clone()));

        let is_reply = email.in_reply_to.map(|v| !v.is_empty()).unwrap_or(false);

        let meta = if entry.has_gmail_meta() {
            gmail_meta::parse_meta(&entry).ok().map(|e| e.into())
        } else {
            None
        };

        // This is filled out at a later stage
        let is_send = false;

        Ok(EmailEntry {
            path: path.to_path_buf(),
            sender_domain,
            sender_local_part,
            sender_name,
            datetime,
            subject,
            meta,
            is_reply,
            to_count,
            to_group,
            to_first,
            is_send,
        })
    }
}

/// Returns a conversion from address to the fields we care about:
/// ([group name], display name, email address)
fn address_to_name_string(address: &Address) -> Option<(Option<String>, String, String)> {
    match address {
        Address::Group((names, boxes)) => match (names.first(), boxes.first()) {
            (group_name, Some(mailbox)) => {
                let group = group_name.map(|e| e.to_string());
                let (display_name, address, _, _) = mailbox_to_string(&mailbox);
                Some((group, display_name, address))
            }
            _ => None,
        },
        Address::Mailbox(mailbox) => {
            let (display_name, address, _, _) = mailbox_to_string(&mailbox);
            Some((None, display_name, address))
        }
    }
}

/// Returns (display name, email address, local part, domain)
fn mailbox_to_string(mailbox: &Mailbox) -> (String, String, String, String) {
    let names = match mailbox.name.as_ref() {
        Some(n) => n
            .iter()
            .map(|e| e.as_ref())
            .collect::<Vec<&str>>()
            .join(" "),
        None => "".to_owned(),
    };
    (
        names,
        emailaddress_to_string(&mailbox.address),
        mailbox.address.local_part.to_string(),
        mailbox.address.domain.to_string(),
    )
}

fn emailaddress_to_string(address: &EmailAddress) -> String {
    format!(
        "{}@{}",
        address.local_part.to_string(),
        address.domain.to_string()
    )
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
