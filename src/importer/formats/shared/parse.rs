use chrono::prelude::*;
use email_parser::address::{Address, EmailAddress, Mailbox};
use eyre::{eyre, Result};

use std::borrow::Cow;
use std::path::Path;

use super::email::{EmailEntry, EmailMeta};

/// Different `importer`s can implement this trait to provide the necessary
/// data to parse their data into a `EmailEntry`.
pub trait ParseableEmail: Send + Sized + Sync {
    /// This will be called once before `message`, `path` and `meta`
    /// are called. It can be used to perform parsing operations
    fn prepare(&mut self) -> Result<()>;
    /// The message content as bytes
    fn message<'a>(&'a self) -> Result<Cow<'a, [u8]>>;
    /// The original path of the email in the filesystem
    fn path(&self) -> &Path;
    /// Optional meta information if they're available.
    /// (Depending on the `importer` capabilities and system)
    fn meta(&self) -> Result<Option<EmailMeta>>;
}

pub fn parse_email<Entry: ParseableEmail>(
    entry: &mut Entry,
    config_sender_email: &str,
) -> Result<EmailEntry> {
    if let Err(e) = entry.prepare() {
        tracing::error!("Prepare Error: {:?}", e);
        return Err(e);
    }
    let content = entry.message()?;
    match email_parser::email::Email::parse(&content) {
        Ok(email) => {
            let path = entry.path();
            tracing::trace!("Parsing {}", path.display());
            let (sender_name, _, sender_local_part, sender_domain) =
                mailbox_to_string(&email.sender);

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

            // In order to determine the sender, we have to
            // build up the address again :-(
            let is_send =
                format!("{}@{}", sender_local_part, sender_domain).as_str() == config_sender_email;

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
        Err(error) => {
            let error = eyre!(
                "Could not parse email (trace to see contents): {:?} [{}]",
                &error,
                entry.path().display()
            );
            tracing::error!("{:?}", &error);
            if let Ok(content_string) = String::from_utf8(content.into_owned()) {
                tracing::trace!("Contents:\n{}\n---\n", content_string);
            } else {
                tracing::trace!("Contents:\nInvalid UTF8\n---\n");
            }
            Err(error)
        }
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
