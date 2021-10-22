use chrono::Datelike;
use crossbeam_channel::{unbounded, Sender};
use eyre::{bail, Report, Result};
use rusqlite::{self, params, Connection, Statement};

use core::panic;
use std::{path::Path, thread::JoinHandle};

use super::{query::Query, query_result::QueryResult, sql::*, DBMessage};
use crate::{database::RowConversion, importer::EmailEntry};

#[derive(Debug)]
pub struct Database {
    connection: Option<Connection>,
}

impl Database {
    /// Open database at path `Path`.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        // FIXME: if the file exists, we're re-opening it.
        // this means we need to query the `meta` table
        // to retrieve the contents of the config...

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

    pub fn query(&self, query: &super::query::Query) -> Result<Vec<QueryResult>> {
        use rusqlite::params_from_iter;
        let c = match &self.connection {
            Some(n) => n,
            None => bail!("No connection to database available in query"),
        };
        let (sql, values) = query.to_sql();
        let mut stmt = c.prepare(&sql)?;
        let mut query_results = Vec::new();
        let mut converted = Vec::new();
        for value in values {
            converted.push(super::conversion::json_to_value(&value)?);
        }

        let p = params_from_iter(converted.iter());

        let mut rows = stmt.query(p)?;
        while let Some(row) = rows.next()? {
            match query {
                Query::Grouped { group_by, .. } => {
                    let result = QueryResult::grouped_from_row(group_by, row)?;
                    query_results.push(result);
                }
                Query::Normal { fields, .. } => {
                    let result = QueryResult::from_row(fields, row)?;
                    query_results.push(result);
                }
            }
        }
        Ok(query_results)
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
                            DBMessage::Error(report) => insert_error(&mut error_prepared, &report),
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
        connection.execute(TBL_META, params![])?;
        Ok(())
    }
}

fn insert_mail(statement: &mut Statement, entry: &EmailEntry) -> Result<()> {
    let path = entry.path.display().to_string();
    let year = entry.datetime.date().year();
    let month = entry.datetime.date().month();
    let day = entry.datetime.date().day();
    let timestamp = entry.datetime.timestamp();
    let e = entry;
    let to_name = e.to_first.as_ref().map(|e| &e.0);
    let to_address = e.to_first.as_ref().map(|e| &e.1);
    let meta_tags = e.meta.as_ref().map(|e| e.tags_string());
    let meta_is_seen = e.meta.as_ref().map(|e| e.is_seen);
    let p = params![
        path,
        e.sender_domain,
        e.sender_local_part,
        e.sender_name,
        year,
        month,
        day,
        timestamp,
        e.subject,
        e.to_count,
        e.to_group,
        to_name,
        to_address,
        e.is_reply,
        e.is_send,
        meta_tags,
        meta_is_seen
    ];
    statement.execute(p)?;
    tracing::trace!("Insert Mail {}", &path);
    Ok(())
}

fn insert_error(statement: &mut Statement, message: &Report) -> Result<()> {
    statement.execute(params![message.to_string()])?;
    tracing::trace!("Insert Error {}", message);
    Ok(())
}
