use super::email::{EmailEntry, EmailMeta};
use super::parse::{parse_email, ParseableEmail};
use crate::database::{DBMessage, Database};
use crate::types::Config;

use super::super::{Message, MessageSender};

use chrono::prelude::*;
use email_parser::address::{Address, EmailAddress, Mailbox};
use eyre::{bail, eyre, Result};
use rayon::prelude::*;
use std::thread::JoinHandle;

use std::convert::{TryFrom, TryInto};
use std::path::Path;

pub fn into_database<Mail: ParseableEmail + 'static>(
    config: &Config,
    emails: Vec<Mail>,
    tx: MessageSender,
) -> Result<usize> {
    let total = emails.len();
    tracing::info!("Loaded {} emails", &total);

    // First, communicate the total amount of mails received
    if let Err(e) = tx.send(Message::WriteTotal(total)) {
        bail!("Channel Failure {:?}", &e);
    }

    // Create a new database connection
    let database = Database::new(config.database_path.clone())?;

    // Consume the connection to begin the import. It will return the `handle` to use for
    // waiting for the database to finish importing, and the `sender` to submit work.
    let (sender, handle) = database.import();

    // Iterate over the mails..
    emails
        // in paralell..
        .par_iter()
        // parsing them
        .map(|raw_mail| (raw_mail.path(), parse_email(raw_mail)))
        // and inserting them into SQLite
        .for_each(|(path, entry)| {
            if let Err(e) = tx.send(Message::WriteOne) {
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

    if let Err(e) = tx.send(Message::FinishingUp) {
        bail!("Channel Failure {:?}", &e);
    }

    tracing::trace!("Waiting for database handle...");
    let output = match handle.join() {
        Ok(Ok(count)) => Ok(count),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(eyre::eyre!("Join Error: {:?}", &e)),
    };

    // Tell the caller that we're done processing. This will allow leaving the
    // display loop
    tracing::trace!("Messaging Done");
    if let Err(e) = tx.send(Message::Done) {
        bail!("Channel Failure {:?}", &e);
    }

    output
}
