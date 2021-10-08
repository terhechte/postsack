mod apple_mail;
mod gmailbackup;
mod importer;
pub mod shared;
use shared::parse::ParseableEmail;

/// This is implemented by the various formats
/// to define how they return email data.
pub trait ImporterFormat {
    type Item: ParseableEmail;
}
