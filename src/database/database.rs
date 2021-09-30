use chrono::Datelike;
use crossbeam_channel::{unbounded, Sender};
use eyre::{Report, Result};
use rusqlite::{self, params, Connection, Statement};

use std::{
    path::{Path, PathBuf},
    thread::JoinHandle,
};

use super::{sql::*, DBMessage};
use crate::types::EmailEntry;

#[derive(Debug)]
pub struct Database {
    connection: Option<Connection>,
}

impl Database {
    /// Open database at path `Path`.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        #[allow(unused_mut)]
        let mut connection = Connection::open(path.as_ref())?;

        // Improve the insertion performance.
        connection.pragma_update(None, "journal_mode", &"memory")?;
        connection.pragma_update(None, "synchronous", &"OFF")?;

        Self::create_tables(&connection)?;

        #[cfg(feature = "trace-sql")]
        connection.trace(Some(|query| {
            tracing::trace!("SQL: {}", &query);
        }));

        Ok(Database {
            connection: Some(connection),
        })
    }

    /// Begin the data import.
    /// This will consume the `Database`. A new one has to be opened
    /// afterwards in order to support multi-threading.
    /// Returns an input `Sender` and a `JoinHandle`.
    /// The `Sender` is used to submit work to the database via `DBMessage`
    /// cases. The `JoinHandle` is used to wait for database completion.
    ///
    /// # Examples
    ///
    /// ```
    /// let db = Database::new("db.sqlite").unwrap();
    /// let (sender, handle) = db.import();
    /// sender.send(DBMessage::Mail(m1)).unwrap();
    /// sender.send(DBMessage::Mail(m2)).unwrap();
    /// handle.join().unwrap();
    /// ```
    pub fn import(mut self) -> (Sender<DBMessage>, JoinHandle<Result<usize>>) {
        let (sender, receiver) = unbounded();

        // Import can only be called *once* on a database created with `new`.
        // Therefore there should always be a value to unwrap;
        let mut connection = self.connection.take().unwrap();
        let handle = std::thread::spawn(move || {
            let mut counter = 0;
            {
                let transaction = connection.transaction()?;
                {
                    let mut mail_prepared = transaction.prepare(QUERY_EMAILS)?;
                    let mut error_prepared = transaction.prepare(QUERY_ERRORS)?;
                    loop {
                        let next = match receiver.recv() {
                            Ok(n) => n,
                            Err(e) => {
                                println!("Receiver error: {:?}", &e);
                                panic!("should not happen");
                            }
                        };
                        match next {
                            DBMessage::Mail(mail) => {
                                counter += 1;
                                insert_mail(&mut mail_prepared, &mail)
                            }
                            DBMessage::Error(report, path) => {
                                insert_error(&mut error_prepared, &report, &path)
                            }
                            DBMessage::Done => {
                                tracing::trace!("Received DBMessage::Done");
                                break;
                            }
                        }?;
                    }
                }
                if let Err(e) = transaction.commit() {
                    return Err(eyre::eyre!("Transaction Error: {:?}", &e));
                }
            }
            // In case closing the database fails, we try again until we succeed
            let mut c = connection;
            loop {
                tracing::trace!("Attempting close");
                match c.close() {
                    Ok(_n) => break,
                    Err((a, _b)) => c = a,
                }
            }
            tracing::trace!("Finished SQLITE: {}", &counter);
            Ok(counter)
        });
        (sender, handle)
    }

    fn create_tables(connection: &Connection) -> Result<()> {
        connection.execute(TBL_EMAILS, params![])?;
        connection.execute(TBL_ERRORS, params![])?;
        Ok(())
    }
}

fn insert_mail(statement: &mut Statement, entry: &EmailEntry) -> Result<()> {
    let path = entry.path.display().to_string();
    let domain = &entry.domain;
    let local_part = &entry.local_part;
    let year = entry.datetime.date().year();
    let month = entry.datetime.date().month();
    let day = entry.datetime.date().day();
    let subject = entry.subject.to_string();
    statement.execute(params![path, domain, local_part, year, month, day, subject])?;
    tracing::trace!("Insert Mail {}", &path);
    Ok(())
}

fn insert_error(statement: &mut Statement, message: &Report, path: &PathBuf) -> Result<()> {
    statement.execute(params![message.to_string(), path.display().to_string()])?;
    tracing::trace!("Insert Error {}", &path.display());
    Ok(())
}
