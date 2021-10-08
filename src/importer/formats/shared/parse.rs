use chrono::prelude::*;
use email_parser::address::{Address, EmailAddress, Mailbox};
use eyre::{bail, eyre, Result};

use std::borrow::{Borrow, Cow};
use std::convert::{TryFrom, TryInto};
use std::path::Path;

use super::email::{EmailEntry, EmailMeta};

/// Different `importer`s can implement this trait to provide the necessary
/// data to parse their data into a `EmailEntry`.
pub trait ParseableEmail: Send + Sized + Sync {
    /// The message content as bytes
    fn message<'a>(&'a self) -> Result<Cow<'a, [u8]>>;
    /// The original path of the email in the filesystem
    fn path(&self) -> &Path;
    /// Optional meta information if they're available.
    /// (Depending on the `importer` capabilities and system)
    fn meta(&self) -> Result<Option<EmailMeta>>;
}

pub fn parse_email<Entry: ParseableEmail>(entry: &Entry) -> Result<EmailEntry> {
    let content = entry.message()?;
    parse_email_parser(entry, content.borrow())
}

fn parse_email_parser<Entry: ParseableEmail>(
    raw_entry: &Entry,
    content: &[u8],
) -> Result<EmailEntry> {
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

impl<'a, Entry: ParseableEmail> TryFrom<(&Entry, email_parser::email::Email<'a>)> for EmailEntry {
    type Error = eyre::Report;
    fn try_from(content: (&Entry, email_parser::email::Email)) -> Result<Self, Self::Error> {
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

        let meta = entry.meta()?;

        // FIXME: This is filled out at a later stage
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
