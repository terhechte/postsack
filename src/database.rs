use std::path::PathBuf;

use crate::emails::EmailEntry;
use chrono::Datelike;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eyre::{Report, Result};
use rusqlite::{self, params, Connection, Error, Row};

#[derive(Debug)]
pub struct Database {
    connection: Connection,
}

/*pub enum DBMessage<'a> {
    Mail(EmailEntry<'a>),
    Error(Report, PathBuf),
    Done,
}*/

impl Database {
    /// Create a in-memory db.
    pub fn new() -> Result<Self> {
        //let mut connection = Connection::open_in_memory()?;
        let mut connection = Connection::open("/tmp/db.sql")?;
        Self::create_tables(&connection)?;
        //connection.trace(Some(|n| {
        //    println!("SQL: {}", &n);
        //}));
        Ok(Database {
            connection: connection,
        })
    }

    pub fn insert_mail(&self, entry: &EmailEntry) -> Result<()> {
        let path = entry.path.display().to_string();
        let domain = &entry.domain;
        let local_part = &entry.local_part;
        let year = entry.datetime.date().year();
        let month = entry.datetime.date().month();
        let day = entry.datetime.date().day();
        let kind = entry.parser.to_string();
        let subject = entry.subject.to_string();
        let sql = r#"INSERT INTO emails (path, domain, local_part, year, month, day, kind, subject) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#;
        let mut prepared = self.connection.prepare(sql)?;
        prepared.execute(params![
            path, domain, local_part, year, month, day, kind, subject
        ])?;
        Ok(())
    }

    pub fn insert_error(&self, message: &Report, path: &PathBuf) -> Result<()> {
        let sql = "INSERT INTO errors (message, path) VALUES (?, ?)";
        let mut prepared = self.connection.prepare(sql)?;
        prepared.execute(params![message.to_string(), path.display().to_string()])?;
        Ok(())
    }

    /*pub fn process(&mut self) -> Sender<DBMessage<'static>> {
        let (sender, receiver) = unbounded();
        let connection = self.connection.take().unwrap();
        std::thread::spawn(move || loop {
            let next = match receiver.recv() {
                Ok(n) => n,
                Err(e) => {
                    println!("Receiver error: {:?}", &e);
                    std::process::exit(0);
                }
            };
            let result = match next {
                DBMessage::Mail(mail) => insert_mail(&connection, &mail),
                DBMessage::Error(report, path) => insert_error(&connection, &report, &path),
                DBMessage::Done => break,
            };
            result.unwrap();
            //if let Err(e) = result {
            //    tracing::error!("SQL Error: {:?}", &e);
            //}
        });
        sender
    }*/

    fn create_tables(connection: &Connection) -> Result<()> {
        let emails_table = r#"
CREATE TABLE IF NOT EXISTS emails (
  path TEXT NOT NULL,
  domain TEXT NOT NULL,
  local_part TEXT NOT NULL,
  year INTEGER NOT NULL,
  month INTEGER NOT NULL,
  day INTEGER NOT NULL,
  kind TEXT NOT NULL,
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
