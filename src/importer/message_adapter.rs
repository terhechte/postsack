use eyre::{bail, eyre, Result};

use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

use super::formats::ImporterFormat;
use super::importer::Importerlike;
use super::Message;

#[derive(Clone, Debug, Copy, Default)]
struct Data {
    total_read: usize,
    read: usize,
    total_write: usize,
    write: usize,
    finishing: bool,
    done: bool,
}

#[derive(Clone, Debug, Copy)]
pub struct Progress {
    pub total: usize,
    pub count: usize,
}

#[derive(Clone, Debug, Copy)]
pub struct State {
    pub finishing: bool,
    pub done: bool,
}

/// This can be initialized with a [`MessageSender`] and it will
/// automatically tally up the information into a thread-safe
/// datastructure
pub struct Adapter {
    producer_lock: Arc<RwLock<Data>>,
    consumer_lock: Arc<RwLock<Data>>,
}

impl Adapter {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let rw_lock = Arc::new(RwLock::default());
        // FIXME: Look up this warning. It looks like the clones are necessary?
        #[allow(clippy::redundant_clone)]
        let producer_lock = rw_lock.clone();
        #[allow(clippy::redundant_clone)]
        let consumer_lock = rw_lock.clone();
        Self {
            producer_lock,
            consumer_lock,
        }
    }

    /// Starts up a thread that handles the `MessageReceiver` messages
    /// into state that can be accessed via [`read_count`], [`write_count`] and [`finished`]
    pub fn process<Format: ImporterFormat + 'static>(
        &self,
        importer: super::importer::Importer<Format>,
    ) -> Result<JoinHandle<Result<()>>> {
        let (receiver, handle) = importer.import()?;
        let lock = self.producer_lock.clone();
        let handle = std::thread::spawn(move || {
            'outer: loop {
                let mut write_guard = match lock.write() {
                    Ok(n) => n,
                    Err(e) => bail!("RwLock Error: {:?}", e),
                };
                for entry in receiver.try_iter() {
                    match entry {
                        Message::ReadTotal(n) => write_guard.total_read = n,
                        Message::ReadOne => write_guard.read += 1,
                        Message::WriteTotal(n) => write_guard.total_write = n,
                        Message::WriteOne => write_guard.write += 1,
                        Message::FinishingUp => write_guard.finishing = true,
                        Message::Done => {
                            write_guard.done = true;
                            break 'outer;
                        }
                    };
                }
            }

            let _ = handle.join().map_err(|op| eyre::eyre!("{:?}", &op))??;

            Ok(())
        });
        Ok(handle)
    }

    pub fn read_count(&self) -> Result<Progress> {
        let item = self.consumer_lock.read().map_err(|e| eyre!("{:?}", &e))?;
        Ok(Progress {
            total: item.total_read,
            count: item.read,
        })
    }

    pub fn write_count(&self) -> Result<Progress> {
        let item = self.consumer_lock.read().map_err(|e| eyre!("{:?}", &e))?;
        Ok(Progress {
            total: item.total_write,
            count: item.write,
        })
    }

    pub fn finished(&self) -> Result<State> {
        let item = self.consumer_lock.read().map_err(|e| eyre!("{:?}", &e))?;
        Ok(State {
            finishing: item.finishing,
            done: item.done,
        })
    }
}
