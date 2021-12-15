pub(crate) mod formats;
#[allow(clippy::module_inception)]
mod importer;
mod message_adapter;

pub use message_adapter::*;
use ps_core::{Config, EmailEntry, EmailMeta, FormatType, Importerlike};

use formats::ImporterFormat;

// pub fn importer(config: &Config) -> Box<dyn Importerlike> {
//     match config.format {
//         AppleMail => Box::new(applemail_importer(config.clone())),
//         GmailVault => Box::new(gmail_importer(config.clone())),
//         Mbox => Box::new(gmail_importer(config.clone())),
//     }
// }

pub fn gmail_importer(config: Config) -> importer::Importer<formats::Gmail> {
    importer::Importer::new(config, formats::Gmail::default())
}

pub fn applemail_importer(config: Config) -> importer::Importer<formats::AppleMail> {
    importer::Importer::new(config, formats::AppleMail::default())
}

pub fn mbox_importer(config: Config) -> importer::Importer<formats::Mbox> {
    importer::Importer::new(config, formats::Mbox::default())
}
