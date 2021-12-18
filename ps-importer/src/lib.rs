use ps_core::eyre::Result;

pub(crate) mod formats;

use formats::{shared, ImporterFormat};

use std::{path::PathBuf, thread::JoinHandle};

use ps_core::{
    crossbeam_channel::unbounded, Config, DatabaseLike, FormatType, Importerlike, Message,
    MessageReceiver,
};

pub struct Importer<Format: ImporterFormat> {
    config: Config,
    format: Format,
}

impl<Format: ImporterFormat + 'static> Importer<Format> {
    pub fn new(config: Config, format: Format) -> Self {
        Self { config, format }
    }
}

impl<Format: ImporterFormat + 'static> Importerlike for Importer<Format> {
    fn import<Database: DatabaseLike + 'static>(
        self,
        database: Database,
    ) -> Result<(MessageReceiver, JoinHandle<Result<()>>)> {
        let Importer { format, .. } = self;
        let (sender, receiver) = unbounded();

        let config = self.config;
        let handle: JoinHandle<Result<()>> = std::thread::spawn(move || {
            let outer_sender = sender.clone();
            let processed = move || {
                let emails = format.emails(&config, sender.clone())?;
                let processed =
                    shared::database::into_database(&config, emails, sender.clone(), database)?;

                Ok(processed)
            };
            let result = processed();

            // Send the error away and map it to a crossbeam channel error
            match result {
                Ok(_) => Ok(()),
                Err(e) => match outer_sender.send(Message::Error(e)) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ps_core::eyre::Report::new(e)),
                },
            }
        });
        Ok((receiver, handle))
    }
}

pub fn gmail_importer(config: Config) -> Importer<formats::Gmail> {
    Importer::new(config, formats::Gmail::default())
}

pub fn applemail_importer(config: Config) -> Importer<formats::AppleMail> {
    Importer::new(config, formats::AppleMail::default())
}

pub fn mbox_importer(config: Config) -> Importer<formats::Mbox> {
    Importer::new(config, formats::Mbox::default())
}

pub fn default_path(format: &FormatType) -> Option<PathBuf> {
    match format {
        FormatType::AppleMail => formats::AppleMail::default_path(),
        FormatType::GmailVault => formats::Gmail::default_path(),
        FormatType::Mbox => formats::Mbox::default_path(),
    }
}
