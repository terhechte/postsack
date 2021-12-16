use eyre::Result;

pub(crate) mod formats;

use formats::{shared, ImporterFormat};

use std::thread::JoinHandle;

use ps_core::{
    crossbeam_channel::{self, unbounded},
    Config, DatabaseLike, EmailEntry, EmailMeta, FormatType, Importerlike, Message,
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
                    Err(e) => Err(eyre::Report::new(e)),
                },
            }
        });
        Ok((receiver, handle))
    }
}

// FIXME:
// impl<T: Importerlike + Sized> Importerlike for Box<T> {
//     fn import(self) -> Result<(MessageReceiver, JoinHandle<Result<()>>)> {
//         (*self).import()
//     }
// }

// pub fn importer(config: &Config) -> Box<dyn Importerlike> {
//     match config.format {
//         AppleMail => Box::new(applemail_importer(config.clone())),
//         GmailVault => Box::new(gmail_importer(config.clone())),
//         Mbox => Box::new(gmail_importer(config.clone())),
//     }
// }

pub fn gmail_importer(config: Config) -> Importer<formats::Gmail> {
    Importer::new(config, formats::Gmail::default())
}

pub fn applemail_importer(config: Config) -> Importer<formats::AppleMail> {
    Importer::new(config, formats::AppleMail::default())
}

pub fn mbox_importer(config: Config) -> Importer<formats::Mbox> {
    Importer::new(config, formats::Mbox::default())
}
