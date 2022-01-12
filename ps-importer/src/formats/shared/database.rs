use super::parse::{parse_email, MessageKind, ParseableEmail};
use ps_core::{Config, DBMessage, DatabaseLike, Message, MessageSender};

use ps_core::eyre::{self, bail, Result};
use ps_core::tracing;
use rayon::prelude::*;

pub fn into_database<Mail: ParseableEmail + 'static, Database: DatabaseLike + 'static>(
    config: &Config,
    mut emails: Vec<Mail>,
    tx: MessageSender,
    database: Database,
) -> Result<usize> {
    let total = emails.len();
    tracing::info!("Loaded {} emails", &total);

    // First, communicate the total amount of mails received
    if let Err(e) = tx.send(Message::WriteTotal(total)) {
        bail!("Channel Failure {:?}", &e);
    }

    // Save the config into the database
    if let Err(e) = database.save_config(config.clone()) {
        bail!("Could not save config to database {:?}", &e);
    }

    // Consume the connection to begin the import. It will return the `handle` to use for
    // waiting for the database to finish importing, and the `sender` to submit work.
    let (sender, handle) = database.import();

    // Iterate over the mails..
    emails
        // in paralell..
        .par_iter_mut()
        // parsing them
        .map(|raw_mail| {
            raw_mail.prepare()?;
            match raw_mail.kind() {
                MessageKind::Data(data) => parse_email(
                    &data,
                    raw_mail.path(),
                    raw_mail.meta()?,
                    &config.sender_emails,
                ),
                MessageKind::Parsed(mail) => Ok(mail),
                MessageKind::Error(e) => Err(e),
            }
        })
        // and inserting them into SQLite
        .for_each(|entry| {
            // Try to write the message into the database
            if let Err(e) = match entry {
                Ok(mail) => sender.send(DBMessage::Mail(Box::new(mail))),
                Err(e) => sender.send(DBMessage::Error(e)),
            } {
                tracing::info!("Error Inserting into Database: {:?}", &e);
                return;
            }
            // Signal the write
            if let Err(e) = tx.send(Message::WriteOne) {
                tracing::info!("Channel Failure: {:?}", &e);
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
