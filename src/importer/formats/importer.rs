use super::{shared, Config, ImporterFormat};

use super::{Message, MessageReceiver, MessageSender};

use crossbeam_channel::{self, unbounded, Receiver, Sender};
use eyre::{Report, Result};
use std::thread::JoinHandle;

pub struct Importer<'a, Format: ImporterFormat> {
    config: &'a Config,
    format: Format,
}

impl<'a, Format: ImporterFormat + 'static> Importer<'a, Format> {
    pub fn new(config: &'a Config, format: Format) -> Self {
        Self { config, format }
    }

    pub fn import(self) -> Result<(MessageReceiver, JoinHandle<Result<usize>>)> {
        let Importer { format, .. } = self;
        let (sender, receiver) = unbounded();

        let config = self.config.clone();
        let handle: JoinHandle<Result<usize>> = std::thread::spawn(move || {
            let emails = format.emails(&config, sender.clone())?;
            let processed = shared::database::into_database(&config, emails, sender.clone())?;

            Ok(processed)
        });
        Ok((receiver, handle))
    }
}
