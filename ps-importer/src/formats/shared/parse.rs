use email_address_parser::EmailAddress;
use mail_parser::{self, Addr, HeaderValue};
use ps_core::chrono;
use ps_core::chrono::prelude::*;
use ps_core::eyre::{eyre, Report, Result};
use ps_core::tracing;

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::Path;

use ps_core::{EmailEntry, EmailMeta};

/// Different `importer`s can implement this trait to provide the necessary
/// data to parse their data into a `EmailEntry`.
pub trait ParseableEmail: Send + Sized + Sync {
    /// This will be called once before `message`, `path` and `meta`
    /// are called. It can be used to perform parsing operations
    fn prepare(&mut self) -> Result<()>;
    /// The message, either as raw bytes for already parsed.
    /// If the importer supports getting the data this has the benefit
    /// of being parsed concurrently already. Some importers types might already
    /// return a fully parsed mail in which case it is easier to
    /// just use the parsed type instead of parsing it all again
    fn kind(&self) -> MessageKind<'_>;
    //fn message(&self) -> Result<Cow<'_, [u8]>>;
    /// The original path of the email in the filesystem
    fn path(&self) -> &Path;
    /// Optional meta information if they're available.
    /// (Depending on the `importer` capabilities and system)
    fn meta(&self) -> Result<Option<EmailMeta>>;
}

#[derive(Debug)]
pub enum MessageKind<'a> {
    Data(Cow<'a, [u8]>),
    #[allow(unused)]
    Parsed(ps_core::EmailEntry),
    Error(Report),
}

pub fn parse_email(
    data: &[u8],
    path: &Path,
    meta: Option<EmailMeta>,
    sender_emails: &HashSet<String>,
) -> Result<EmailEntry> {
    match mail_parser::Message::parse(&data) {
        Some(email) => {
            tracing::info!("Parsing {}", path.display());

            // For the `sender` we take the sender field and if that doesn't exist the `from` field.
            // This allows also parsing mailing list message dumps where sender -> from.
            let sender_header = email.get_sender();
            let (sender_name, address, sender_local_part, sender_domain) = match sender_header {
                HeaderValue::Empty => split_single_address_header(&email.get_from()),
                _ => split_single_address_header(sender_header),
            }
            .ok_or(eyre!("Could not parse address: {:?}", email.get_sender()))?;

            let datetime =
                emaildatetime_to_chrono(email.get_date()).ok_or(eyre!("Could not parse date"))?;

            let subject = email.get_subject().unwrap_or_default().to_string();

            let (to_count, to_group, to_first) =
                split_multi_address_header(email.get_to()).unwrap_or((0, None, None));

            let mut is_reply = false;
            match split_single_address_header(&email.get_reply_to()) {
                Some(_) => is_reply = true,
                None => match split_single_address_header(&email.get_in_reply_to()) {
                    Some(_) => is_reply = true,
                    None => (),
                },
            }

            let is_send = sender_emails.contains(&address);

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
        None => {
            let error = eyre!(
                "Could not parse email (trace to see contents): [{}]",
                path.display()
            );
            if let Ok(content_string) = String::from_utf8(data.to_vec()) {
                tracing::trace!("Contents:\n{}\n---\n", content_string);
            } else {
                tracing::trace!("Contents:\nInvalid UTF8\n---\n");
            }
            Err(error)
        }
    }
}

/// Parse an `Addr` into its constituents
/// Returns (display name, email address, local part, domain)
/// Returns none if the address in the email can't be parsed
fn parse_address(addr: &Addr) -> Option<(String, String, String, String)> {
    let name = addr
        .name
        .as_ref()
        .map(|e| e.to_string())
        .unwrap_or("".to_owned());
    let address = addr.address.as_ref()?.to_string();

    // Parse the address, support invalid mails
    let parsed_address = EmailAddress::parse(&address, None)?;

    let local_part = parsed_address.get_local_part().to_owned();
    let domain = parsed_address.get_domain().to_owned();

    Some((name, address, local_part, domain))
}

/// Returns (display name, email address, local part, domain)
fn split_single_address_header(value: &HeaderValue) -> Option<(String, String, String, String)> {
    let addr = match value {
        HeaderValue::Address(addr) => addr,
        HeaderValue::AddressList(addrs) if !addrs.is_empty() => &addrs[0],
        HeaderValue::Group(grp) if !grp.addresses.is_empty() => &grp.addresses[0],
        HeaderValue::GroupList(grps) if !grps.is_empty() && !&grps[0].addresses.is_empty() => {
            &grps[0].addresses[0]
        }
        _ => {
            tracing::error!("Invalid mail data in address field: {:?}", &value);
            return None;
        }
    };
    parse_address(addr)
}

/// Returns `(amount of to addresses, optional name of to group, (optional address or first address in group, optional name of first address in group))`
fn split_multi_address_header(
    value: &HeaderValue,
) -> Option<(usize, Option<String>, Option<(String, String)>)> {
    let (addr, count, name) = match value {
        HeaderValue::Address(addr) => (addr, 1, None),
        HeaderValue::AddressList(addrs) if !addrs.is_empty() => (&addrs[0], addrs.len(), None),
        HeaderValue::Group(grp) if !grp.addresses.is_empty() => {
            (&grp.addresses[0], grp.addresses.len(), grp.name.as_ref())
        }
        HeaderValue::GroupList(grps) if !grps.is_empty() && !&grps[0].addresses.is_empty() => {
            let total: usize = grps.iter().fold(0, |a, b| a + b.addresses.len());
            (
                &grps[0].addresses[0],
                total,
                grps.first().and_then(|e| e.name.as_ref()),
            )
        }
        _ => {
            tracing::error!("Invalid mail data in address field: {:?}", &value);
            return None;
        }
    };

    let name = name.map(|e| e.to_string());

    let (display_name, address, _, _) = parse_address(addr)?;
    Some((count, name, Some((address, display_name))))
}

fn emaildatetime_to_chrono(
    datetime: Option<&mail_parser::DateTime>,
) -> Option<chrono::DateTime<Utc>> {
    let dt = datetime?;
    Some(
        Utc.ymd(dt.year as i32, dt.month as u32, dt.day as u32)
            .and_hms(dt.hour as u32, dt.minute as u32, dt.second as u32),
    )
}
