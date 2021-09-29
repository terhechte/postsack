use chrono::prelude::*;
use email_address_parser;
use eyre::{bail, eyre, Result, WrapErr};
use flate2;
use flate2::read::GzDecoder;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use strum_macros;

const SENDER_HEADER_NAMES: &[&str] = &["Sender", "Reply-to", "From"];
const DATE_HEADER_NAMES: &[&str] = &["Received", "Date"];

use std::{
    convert::{TryFrom, TryInto},
    io::Read,
    path::{Path, PathBuf},
};

/// We want to know which library was used to parse this email
#[derive(Debug, strum_macros::EnumString, strum_macros::ToString)]
pub enum ParserKind {
    EmailParser,
    Eml,
    Rhymessage,
    Meta,
}

/// Representation of an email
#[derive(Debug)]
pub struct EmailEntry<'a> {
    pub path: PathBuf,
    pub domain: Cow<'a, str>,
    pub local_part: Cow<'a, str>,
    pub datetime: chrono::DateTime<Utc>,
    pub parser: ParserKind,
    pub subject: Cow<'a, str>,
}

/// Raw representation of an email.
/// Contains the paths to the relevant files as well
/// as the name of the folder the email was in.
#[derive(Debug)]
pub struct RawEmailEntry {
    folder_name: String,
    eml_path: PathBuf,
    meta_path: PathBuf,
}

impl RawEmailEntry {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> RawEmailEntry {
        let path = path.as_ref();
        let folder_name = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let eml_path = path.to_path_buf();
        let meta_path = path
            .parent()
            .unwrap()
            .join(format!(
                "{}.meta",
                &path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace(".eml", "")
            ))
            .to_path_buf();
        RawEmailEntry {
            folder_name,
            eml_path,
            meta_path,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.eml_path.clone()
    }
}

pub struct Emails {
    /// The current index in the Vec of emails
    curr: usize,
    /// The `Vec` with the `EmailEntry` entries
    pub emails: Vec<RawEmailEntry>,
}

impl Emails {
    pub fn new<A: AsRef<Path>>(folder: A) -> Result<Self> {
        let folder = folder.as_ref();
        if !folder.exists() {
            bail!("Folder {} does not exist", &folder.display());
        }
        let emails = read_folders(&folder)?;
        Ok(Emails { curr: 0, emails })
    }

    pub fn len(&self) -> usize {
        self.emails.len()
    }
}

//impl Iterator for Emails {
//    // We can refer to this type using Self::Item
//    type Item = Result<EmailEntry>;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        let new_next = self.curr + 1;
//        let entry = self.emails.get(self.curr)?;
//        self.curr = new_next;
//        let email = read_email(&entry);
//        Some(email)
//    }
//}

//impl ParallelIterator for Emails {
//    type Item = Result<EmailEntry>;
//
//    fn drive_unindexed<C>(self, consumer: C) -> C::Result
//    where
//        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
//    {
//        self.emails
//            .into_par_iter()
//            .map(|e| read_email(&e))
//            .drive_unindexed(consumer)
//    }
//}

pub fn read_folders<P: AsRef<Path>>(folder: &P) -> Result<Vec<RawEmailEntry>> {
    let folder = folder.as_ref();
    Ok(std::fs::read_dir(&folder)?
        .into_iter()
        .par_bridge()
        .filter_map(|entry| {
            let path = entry
                .map_err(|e| tracing::error!("{} {:?}", &folder.display(), &e))
                .ok()?
                .path();
            if !path.is_dir() {
                return None;
            }
            read_emails(&path)
                .map_err(|e| tracing::error!("{} {:?}", &path.display(), &e))
                .ok()
        })
        .flatten()
        .collect())
}

fn read_emails(folder_path: &Path) -> Result<Vec<RawEmailEntry>> {
    Ok(std::fs::read_dir(folder_path)?
        .into_iter()
        .par_bridge()
        .filter_map(|entry| {
            let path = entry
                .map_err(|e| tracing::error!("{} {:?}", &folder_path.display(), &e))
                .ok()?
                .path();
            if path.is_dir() {
                return None;
            }
            if !path.extension()?.eq("gz") {
                return None;
            }
            Some(RawEmailEntry {
                folder_name: folder_path.file_name()?.to_str()?.to_string(),
                eml_path: path.clone(),
                meta_path: path
                    .parent()?
                    .join(format!(
                        "{}.meta",
                        &path.file_stem()?.to_str()?.replace(".eml", "")
                    ))
                    .to_path_buf(),
            })
        })
        .collect())
}

type ProgressSender = Sender<Result<Option<usize>>>;

use crate::database::Database;
use crossbeam_channel::Sender;
use crossbeam_deque::{Stealer, Worker};
pub fn process_emails(
    emails: Vec<RawEmailEntry>,
    db: Arc<Mutex<Database>>,
    progress_sender: ProgressSender,
) {
    // This will open the worker thread that supplies the stealers
    // with RawEmailEntry instances
    let worker: Worker<RawEmailEntry> = Worker::new_fifo();
    let cpus = num_cpus::get() * 2;
    let handles: Vec<JoinHandle<Result<()>>> = (0..=cpus)
        .map(|_| {
            let stealer = worker.stealer();
            let sender = progress_sender.clone();
            let db = db.clone();
            std::thread::spawn(move || {
                if let Err(e) = stealer_thread(stealer, db, sender) {
                    // FIXME: Report back
                    println!("{}", &e);
                    //db.lock().map(|e| e.insert_error(e, path))
                }
                Ok(())
            })
        })
        .collect();

    for email in emails {
        worker.push(email);
    }

    for handle in handles {
        let _ = handle.join().unwrap();
    }

    progress_sender.send(Ok(None));
}

fn stealer_thread(
    stealer: Stealer<RawEmailEntry>,
    db: Arc<Mutex<Database>>,
    progress_sender: ProgressSender,
) -> Result<()> {
    while let crossbeam_deque::Steal::Success(raw_entry) = stealer.steal() {
        let content = match unziped_content(&raw_entry.eml_path) {
            Ok(n) => n,
            Err(_) => continue,
        };
        let email = match read_email(&raw_entry.eml_path.as_path(), &content) {
            Ok(n) => n,
            Err(_) => continue,
        };

        let result = db.lock().map(|e| e.insert_mail(&email));
        progress_sender.send(Ok(Some(1)));
        match result {
            Ok(Err(e)) => {
                println!("Insertion Error: {}", &e)
            }
            Ok(Ok(_)) => (),
            Err(e) => {
                println!("Poison Guard: {:?}", &e);
            }
        }
    }

    Ok(())
}

pub fn read_email<'a, 'b>(path: &'b Path, content: &'a [u8]) -> Result<EmailEntry<'a>> {
    match email_parser::email::Email::parse(&content) {
        Ok(email) => {
            let domain = email.sender.address.domain;
            let local_part = email.sender.address.local_part;
            let datetime = emaildatetime_to_chrono(&email.date);
            let subject = email.subject.unwrap_or_default();

            let entry = EmailEntry {
                path: path.to_path_buf(),
                domain,
                local_part,
                datetime,
                parser: ParserKind::EmailParser,
                subject,
            };
            Ok(entry)
        }
        Err(error) => Err(eyre!("Could not `email_parser` email:\n{:?}", &error)),
    }
}

/*fn parse_meta(raw_entry: &RawEmailEntry, _content: &Vec<u8>) -> Result<EmailEntry> {
    use chrono::prelude::*;
    #[derive(Deserialize)]
    struct Meta {
        msg_id: String,
        internal_date: i64,
        subject: String,
    }
    let content = std::fs::read_to_string(&raw_entry.meta_path)?;
    let meta: Meta = serde_json::from_str(&content)?;
    let parsed = email_address_parser::EmailAddress::parse(&meta.msg_id, None)
        .ok_or(eyre!("Cannot Parse Address: {}", &meta.msg_id))?;
    let datetime = Utc.timestamp(meta.internal_date, 0);
    Ok(EmailEntry {
        path: raw_entry.eml_path.to_path_buf(),
        domain: parsed.get_domain().to_owned(),
        local_part: parsed.get_local_part().to_owned(),
        datetime,
        parser: ParserKind::Meta,
        subject: meta.subject.clone(),
    })
}*/

/*impl<'a, 'b> TryFrom<(&'b Path, email_parser::email::Email<'a>)> for EmailEntry<'a> {
    type Error = eyre::Report;
    fn try_from(content: (&'b Path, email_parser::email::Email<'a>)) -> Result<Self, Self::Error>
    where
        'b: 'a,
    {
        let (path, email) = content;
        let domain = email.sender.address.domain;
        let local_part = email.sender.address.local_part;
        let datetime = emaildatetime_to_chrono(&email.date);
        let subject = email.subject.unwrap_or_default();

        Ok(EmailEntry {
            path,
            domain,
            local_part,
            datetime,
            parser: ParserKind::EmailParser,
            subject,
        })
    }
}*/

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

fn unziped_content(path: &Path) -> Result<Vec<u8>> {
    let reader = std::fs::File::open(path)?;
    let mut decoder = GzDecoder::new(reader);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Try to parse unstructed data into some sort of
/// email address
//fn parse_unstructured(data: &str) -> Option<eml_parser::eml::EmailAddress> {
//    use lazy_static::lazy_static;
//    use regex::Regex;
//    lazy_static! {
//        static ref EMAIL_RE: Regex = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();
//    }
//    lazy_static! {
//        static ref RE: Regex = Regex::new("<(.*?)>").unwrap();
//    }
//    if let Some(capture) = RE.captures(&data).and_then(|f| f.get(1)) {
//        Some(eml_parser::eml::EmailAddress::AddressOnly {
//            address: capture.as_str().to_string(),
//        })
//    } else {
//        let capture = EMAIL_RE.captures(&data).and_then(|f| f.get(0))?;
//        Some(eml_parser::eml::EmailAddress::AddressOnly {
//            address: capture.as_str().to_string(),
//        })
//    }
//}

//fn extract_address(from: &eml_parser::eml::EmailAddress) -> String {
//    use eml_parser::eml::EmailAddress::*;
//    match from {
//        AddressOnly { address } => address.clone(),
//        NameAndEmailAddress { name: _, address } => address.clone(),
//    }
//}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::RawEmailEntry;

    //#[test]
    //fn test_weird_email1() {
    //    let data = "No Reply <no-reply@evernote.com>, terhechte.5cffa@m.evernote.com";
    //    let address = super::parse_unstructured(&data).unwrap();
    //    assert_eq!(
    //        address,
    //        eml_parser::eml::EmailAddress::AddressOnly {
    //            address: "no-reply@evernote.com".to_owned()
    //        }
    //    );
    //}
    //#[test]
    //fn test_weird_email2() {
    //    let data = r#"info@sport-news.denReply-To:info"@sport-news.denX-Mailer:Sport-News.de"#;
    //    let address = super::parse_unstructured(&data).unwrap();
    //    assert_eq!(
    //        address,
    //        eml_parser::eml::EmailAddress::AddressOnly {
    //            address: "info@sport-news.den".to_owned()
    //        }
    //    );
    //}
    //#[test]
    // fn test_weird_email3() {
    //     crate::setup();
    //     let eml_path = PathBuf::from_str(
    //         "/Users/terhechte/Documents/gmail_backup/db/2014-09/1479692635489080640.eml.gz",
    //     )
    //     .unwrap();
    //     let meta_path = PathBuf::from_str(
    //         "/Users/terhechte/Documents/gmail_backup/db/2014-09/1479692635489080640.meta",
    //     )
    //     .unwrap();
    //     let r = RawEmailEntry {
    //         folder_name: "2014-09".to_owned(),
    //         eml_path,
    //         meta_path,
    //     };
    //     //let result = super::read_email(&r).expect("");
    //     let content = Vec::new();
    //     let result = super::parse_meta(&r, &content).expect("");
    //     dbg!(&result);
    // }

    // #[test]
    // fn test_weird_email4() {
    //     crate::setup();
    //     let eml_path = PathBuf::from_str(
    //         "/Users/terhechte/Documents/gmail_backup/db/2014-08/1475705321427236077.eml.gz",
    //     )
    //     .unwrap();
    //     let meta_path = PathBuf::from_str(
    //         "/Users/terhechte/Documents/gmail_backup/db/2014-08/1475705321427236077.meta",
    //     )
    //     .unwrap();
    //     let r = RawEmailEntry {
    //         folder_name: "2014-08".to_owned(),
    //         eml_path,
    //         meta_path,
    //     };
    //     let result = super::read_email(&r).expect("");
    //     dbg!(&result);
    // }
}
