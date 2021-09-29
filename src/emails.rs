use chrono::prelude::*;
use email_address_parser;
use eml_parser::eml::HeaderFieldValue;
use eyre::{bail, eyre, Result, WrapErr};
use flate2;
use flate2::read::GzDecoder;
use rayon::prelude::*;
use rhymessage;
use serde::Deserialize;
use serde_json;
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
pub struct EmailEntry {
    pub path: PathBuf,
    pub domain: String,
    pub local_part: String,
    pub datetime: chrono::DateTime<Utc>,
    pub parser: ParserKind,
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

fn read_folders(folder: &Path) -> Result<Vec<RawEmailEntry>> {
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

pub fn read_email(raw_entry: &RawEmailEntry) -> Result<EmailEntry> {
    let content = unziped_content(&raw_entry.eml_path)?;
    // We have to try multiple different email readers as each of them seems to fail in a different way
    let email = parse_email_parser(&raw_entry, &content)
        .or_else(|e| {
            tracing::trace!("Parser Error: {:?}", &e);
            parse_rhymessage(&raw_entry, &content)
        })
        .or_else(|e| {
            tracing::trace!("Parser Error: {:?}", &e);
            parse_eml(&raw_entry, &content)
        })
        .or_else(|e| {
            tracing::trace!("Parser Error: {:?}", &e);
            parse_meta(&raw_entry, &content)
        });

    Ok(email.wrap_err_with(|| {
        format!(
            "{}\n{:?}",
            String::from_utf8(content.clone()).unwrap(),
            &raw_entry
        )
    })?)
}

fn parse_email_parser(raw_entry: &RawEmailEntry, content: &Vec<u8>) -> Result<EmailEntry> {
    let x = match email_parser::email::Email::parse(&content) {
        Ok(email) => (&raw_entry.eml_path, email).try_into(),
        Err(error) => {
            let content_string = String::from_utf8(content.clone())?;
            //println!(
            //    "---\n{}\n---\n{:?}\n---\n{}",
            //    &content_string,
            //    &error,
            //    &raw_entry.eml_path.display()
            //);
            println!("{}|{}", &error, &raw_entry.eml_path.display());
            //Err(eyre!("Could not `email_parser` email:\n{:?}", &error))
            Err(eyre!("Could not `email_parser` email"))
        }
    };
    x
    //.unwrap();
    //Ok(x)
}

fn parse_eml(raw_entry: &RawEmailEntry, content: &Vec<u8>) -> Result<EmailEntry> {
    let content_string = String::from_utf8(content.clone())?;
    match eml_parser::EmlParser::from_string(content_string)
        .ignore_body()
        .parse()
    {
        Ok(eml) => (&raw_entry.eml_path, eml).try_into(),
        Err(error) => bail!("Could not `eml` parse email:\n{:?}", &error),
    }
}

fn parse_meta(raw_entry: &RawEmailEntry, _content: &Vec<u8>) -> Result<EmailEntry> {
    use chrono::prelude::*;
    #[derive(Deserialize)]
    struct Meta {
        msg_id: String,
        internal_date: i64,
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
    })
}

fn parse_rhymessage(raw_entry: &RawEmailEntry, content: &Vec<u8>) -> Result<EmailEntry> {
    use rhymessage::MessageHeaders;
    let mut headers = MessageHeaders::new();
    match headers.parse(&content) {
        Ok(_) => (),
        Err(e) => bail!("Error Parsing Message: {:?}", &e),
    }
    Ok((&raw_entry.eml_path, headers).try_into()?)
}

impl<'a> TryFrom<(&PathBuf, email_parser::email::Email<'a>)> for EmailEntry {
    type Error = eyre::Report;
    fn try_from(content: (&PathBuf, email_parser::email::Email)) -> Result<Self, Self::Error> {
        let (path, email) = content;
        let domain = email.sender.address.domain.to_string();
        let local_part = email.sender.address.local_part.to_string();
        let datetime = emaildatetime_to_chrono(&email.date);

        Ok(EmailEntry {
            path: path.to_path_buf(),
            domain,
            local_part,
            datetime,
            parser: ParserKind::EmailParser,
        })
    }
}

impl TryFrom<(&PathBuf, rhymessage::MessageHeaders)> for EmailEntry {
    type Error = eyre::Report;
    fn try_from(content: (&PathBuf, rhymessage::MessageHeaders)) -> Result<Self, Self::Error> {
        let (path, headers) = content;

        let mut address: Option<String> = None;
        let mut date: Option<String> = None;
        for entry in headers.headers() {
            if address == None && SENDER_HEADER_NAMES.contains(&entry.name.as_ref()) {
                address = Some(entry.value.to_string());
            }
            if date == None && DATE_HEADER_NAMES.contains(&entry.name.as_ref()) {
                date = Some(entry.value.to_string());
            }
            if address.is_some() && date.is_some() {
                break;
            }
        }

        let address = address.ok_or(eyre!("Cannot find sender header"))?;
        let date = date.ok_or(eyre!("Cannot find date header"))?;

        let parsed_address = email_address_parser::EmailAddress::parse(&address, None)
            .ok_or(eyre!("Cannot Parse Address: {}", &address))?;

        let parsed_date = date
            .parse::<DateTime<Utc>>()
            .map_err(|e| eyre!("Cannot Parse Date {}: {:?}", &date, &e))?;

        Ok(EmailEntry {
            path: path.to_path_buf(),
            domain: parsed_address.get_domain().to_string(),
            local_part: parsed_address.get_local_part().to_string(),
            datetime: parsed_date,
            parser: ParserKind::Rhymessage,
        })
    }
}

impl TryFrom<(&PathBuf, eml_parser::eml::Eml)> for EmailEntry {
    type Error = eyre::Report;
    fn try_from(content: (&PathBuf, eml_parser::eml::Eml)) -> Result<Self, Self::Error> {
        use eml_parser::eml::EmailAddress;
        let (path, email) = content;
        let headers = email.headers;
        let sender = email
            .from
            .as_ref()
            .or_else(|| {
                // Try to find the address from some other field
                headers
                    .iter()
                    .find(|f| SENDER_HEADER_NAMES.contains(&f.name.as_str()))
                    .map(|f| &f.value)
            })
            .ok_or(eyre!("Missing From Field"))?;

        let datetime = headers
            .iter()
            .find(|f| DATE_HEADER_NAMES.contains(&f.name.as_str()))
            .map(|f| match &f.value {
                HeaderFieldValue::Unstructured(s) => Some(s.clone()),
                _ => None,
            })
            .flatten()
            .ok_or(eyre!("Missing Date Field"))?;

        let parsed_date = datetime
            .parse::<DateTime<Utc>>()
            .map_err(|e| eyre!("Cannot Parse Date {}: {:?}", &datetime, &e))?;

        use eml_parser::eml::HeaderFieldValue::*;
        let address = match &sender {
            SingleEmailAddress(e) => EmailAddress::AddressOnly {
                address: extract_address(e),
            },
            MultipleEmailAddresses(e) if !e.is_empty() => EmailAddress::AddressOnly {
                address: extract_address(e.get(0).unwrap()),
            },
            Unstructured(data) => {
                parse_unstructured(&data).ok_or(eyre!("Invalid Unstructered Email: {}", &data))?
            }
            MultipleEmailAddresses(e) => {
                bail!("Email has invalid amount of senders: {:?}", &e)
            }
            _ => bail!("Email has invalid amount of senders: {:?}", &sender),
        };

        let address = extract_address(&address);

        let parsed = email_address_parser::EmailAddress::parse(&address, None)
            .ok_or(eyre!("Cannot Parse Address: {}", &address))?;

        Ok(EmailEntry {
            path: path.to_path_buf(),
            domain: parsed.get_domain().to_string(),
            local_part: parsed.get_local_part().to_string(),
            datetime: parsed_date,
            parser: ParserKind::Eml,
        })
    }
}

fn emaildatetime_to_chrono(dt: &email_parser::time::DateTime) -> chrono::DateTime<Utc> {
    use email_parser::time::Month::*;
    let m = match dt.date.month {
        January => 1,
        February => 2,
        March => 3,
        April => 4,
        May => 5,
        June => 6,
        July => 7,
        August => 8,
        September => 9,
        October => 10,
        November => 11,
        December => 12,
    };
    Utc.ymd(dt.date.year as i32, m, dt.date.day as u32).and_hms(
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
fn parse_unstructured(data: &str) -> Option<eml_parser::eml::EmailAddress> {
    use lazy_static::lazy_static;
    use regex::Regex;
    lazy_static! {
        static ref EMAIL_RE: Regex = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();
    }
    lazy_static! {
        static ref RE: Regex = Regex::new("<(.*?)>").unwrap();
    }
    if let Some(capture) = RE.captures(&data).and_then(|f| f.get(1)) {
        Some(eml_parser::eml::EmailAddress::AddressOnly {
            address: capture.as_str().to_string(),
        })
    } else {
        let capture = EMAIL_RE.captures(&data).and_then(|f| f.get(0))?;
        Some(eml_parser::eml::EmailAddress::AddressOnly {
            address: capture.as_str().to_string(),
        })
    }
}

fn extract_address(from: &eml_parser::eml::EmailAddress) -> String {
    use eml_parser::eml::EmailAddress::*;
    match from {
        AddressOnly { address } => address.clone(),
        NameAndEmailAddress { name: _, address } => address.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::RawEmailEntry;

    #[test]
    fn test_weird_email1() {
        let data = "No Reply <no-reply@evernote.com>, terhechte.5cffa@m.evernote.com";
        let address = super::parse_unstructured(&data).unwrap();
        assert_eq!(
            address,
            eml_parser::eml::EmailAddress::AddressOnly {
                address: "no-reply@evernote.com".to_owned()
            }
        );
    }

    #[test]
    fn test_weird_email2() {
        let data = r#"info@sport-news.denReply-To:info"@sport-news.denX-Mailer:Sport-News.de"#;
        let address = super::parse_unstructured(&data).unwrap();
        assert_eq!(
            address,
            eml_parser::eml::EmailAddress::AddressOnly {
                address: "info@sport-news.den".to_owned()
            }
        );
    }

    #[test]
    fn test_weird_email3() {
        crate::setup();
        let eml_path = PathBuf::from_str(
            "/Users/terhechte/Documents/gmail_backup/db/2014-09/1479692635489080640.eml.gz",
        )
        .unwrap();
        let meta_path = PathBuf::from_str(
            "/Users/terhechte/Documents/gmail_backup/db/2014-09/1479692635489080640.meta",
        )
        .unwrap();
        let r = RawEmailEntry {
            folder_name: "2014-09".to_owned(),
            eml_path,
            meta_path,
        };
        //let result = super::read_email(&r).expect("");
        let content = Vec::new();
        let result = super::parse_meta(&r, &content).expect("");
        dbg!(&result);
    }

    #[test]
    fn test_weird_email4() {
        crate::setup();
        let eml_path = PathBuf::from_str(
            "/Users/terhechte/Documents/gmail_backup/db/2014-08/1475705321427236077.eml.gz",
        )
        .unwrap();
        let meta_path = PathBuf::from_str(
            "/Users/terhechte/Documents/gmail_backup/db/2014-08/1475705321427236077.meta",
        )
        .unwrap();
        let r = RawEmailEntry {
            folder_name: "2014-08".to_owned(),
            eml_path,
            meta_path,
        };
        let result = super::read_email(&r).expect("");
        dbg!(&result);
    }
}
