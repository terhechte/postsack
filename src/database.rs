use std::{
    path::{Path, PathBuf},
    thread::JoinHandle,
};

use crate::types::EmailEntry;
use chrono::Datelike;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eyre::{Report, Result};
use rusqlite::{self, params, Connection, Error, Row, Statement, Transaction};

#[derive(Debug)]
pub struct Database {
    connection: Option<Connection>,
}

pub enum DBMessage {
    Mail(EmailEntry),
    Error(Report, PathBuf),
    Done,
}

impl Database {
    /// Create a in-memory db.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        //let mut connection = Connection::open_in_memory()?;
        let connection = Connection::open(path.as_ref())?;
        connection
            .pragma_update(None, "journal_mode", &"memory")
            .unwrap();
        connection
            .pragma_update(None, "synchronous", &"OFF")
            .unwrap();
        Self::create_tables(&connection)?;
        //connection.trace(Some(|n| {
        //    println!("SQL: {}", &n);
        //}));
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
        let mut connection = self.connection.take().unwrap();
        let handle = std::thread::spawn(move || {
            let mut counter = 0;
            {
                let transaction = connection.transaction().unwrap();
                let sql = "INSERT INTO emails (path, domain, local_part, year, month, day, subject) VALUES (?, ?, ?, ?, ?, ?, ?)";
                {
                    let mut prepared = transaction.prepare(sql).unwrap();
                    loop {
                        let next = match receiver.recv() {
                            Ok(n) => n,
                            Err(e) => {
                                println!("Receiver error: {:?}", &e);
                                panic!("should not happen");
                            }
                        };
                        let result = match next {
                            DBMessage::Mail(mail) => {
                                counter += 1;
                                insert_mail(&transaction, &mut prepared, &mail)
                            }
                            DBMessage::Error(report, path) => {
                                insert_error(&transaction, &report, &path)
                            }
                            DBMessage::Done => {
                                tracing::trace!("Received DBMessage::Done");
                                break;
                            }
                        };
                        result.unwrap();
                        //if let Err(e) = result {
                        //    tracing::error!("SQL Error: {:?}", &e);
                        //}
                    }
                }
                if let Err(e) = transaction.commit() {
                    return Err(eyre::eyre!("Transaction Error: {:?}", &e));
                }
            }
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
        let emails_table = r#"
CREATE TABLE IF NOT EXISTS emails (
  path TEXT NOT NULL,
  domain TEXT NOT NULL,
  local_part TEXT NOT NULL,
  year INTEGER NOT NULL,
  month INTEGER NOT NULL,
  day INTEGER NOT NULL,
  subject TEXT NOT NULL
);"#;
        connection.execute(&emails_table, params![])?;
        let errors_table = r#"
CREATE TABLE IF NOT EXISTS errors (
  message TEXT NOT NULL,
  path TEXT NOT NULL
);"#;
        connection.execute(&errors_table, params![])?;
        Ok(())
    }
}

fn insert_mail(
    transaction: &Transaction,
    statement: &mut Statement,
    entry: &EmailEntry,
) -> Result<()> {
    let path = entry.path.display().to_string();
    let domain = &entry.domain;
    let local_part = &entry.local_part;
    let year = entry.datetime.date().year();
    let month = entry.datetime.date().month();
    let day = entry.datetime.date().day();
    let subject = entry.subject.to_string();
    let r = statement.execute(params![path, domain, local_part, year, month, day, subject])?;
    tracing::trace!("Insert Mail [{}] {}", r, &path);
    Ok(())
}

fn insert_error(transaction: &Transaction, message: &Report, path: &PathBuf) -> Result<()> {
    let sql = "INSERT INTO errors (message, path) VALUES (?, ?)";
    tracing::trace!("Insert Error {}", &path.display());
    let mut prepared = transaction.prepare(sql)?;
    prepared.execute(params![message.to_string(), path.display().to_string()])?;
    Ok(())
}

pub trait RowConversion: Sized {
    fn from_row<'stmt>(row: &Row<'stmt>) -> Result<Self, Error>;
    fn to_row(&self) -> Result<String, Error>;
}

/*impl RowConversion for EmailEntry {
fn from_row<'stmt>(row: &Row<'stmt>) -> Result<Self, Error> {
    let path: String = row.get("path")?;
    let domain: String = row.get("domain")?;
    let local_part: String = row.get("local_part")?;
    let year: usize = row.get("year")?;
    let month: usize = row.get("month")?;
    let day: usize = row.get("day")?;
    let created = email_parser::time::DateTime::
    Ok(EmailEntry {
        path, domain, local_part, year, month, day
    })
}
*/
