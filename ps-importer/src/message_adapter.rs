use eyre::{bail, eyre, Report, Result};

use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

use super::formats::ImporterFormat;
use ps_core::{DatabaseLike, Importerlike, Message};

#[derive(Debug, Default)]
struct Data {
    total_read: usize,
    read: usize,
    total_write: usize,
    write: usize,
    finishing: bool,
    done: bool,
    error: Option<Report>,
    #[cfg(target_os = "macos")]
    missing_permissions: bool,
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
    pub written: usize,
    #[cfg(target_os = "macos")]
    pub missing_permissions: bool,
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
    pub fn process<Format: ImporterFormat + 'static, Database: DatabaseLike + 'static>(
        &self,
        importer: super::importer::Importer<Format>,
        database: Database,
    ) -> Result<JoinHandle<Result<()>>> {
        let (receiver, handle) = importer.import(database)?;
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
                        Message::ReadOne => {
                            write_guard.read += 1;
                            // Depending on the implementation, we may receive read calls before
                            // the total size is known. We prevent division by zero by
                            // always setting the total to read + 1 in these cases
                            if write_guard.total_read <= write_guard.read {
                                write_guard.total_read = write_guard.read + 1;
                            }
                        }
                        Message::WriteTotal(n) => write_guard.total_write = n,
                        Message::WriteOne => write_guard.write += 1,
                        Message::FinishingUp => write_guard.finishing = true,
                        Message::Done => {
                            write_guard.done = true;
                            break 'outer;
                        }
                        Message::Error(e) => {
                            write_guard.error = Some(e);
                        }
                        #[cfg(target_os = "macos")]
                        Message::MissingPermissions => {
                            write_guard.missing_permissions = true;
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
            written: item.write,
            #[cfg(target_os = "macos")]
            missing_permissions: item.missing_permissions,
        })
    }

    pub fn error(&self) -> Result<Option<Report>> {
        // We take the error of out of the write lock only if there is an error.
        let item = self.consumer_lock.read().map_err(|e| eyre!("{:?}", &e))?;
        let is_error = item.error.is_some();
        drop(item);
        if is_error {
            let mut item = self.producer_lock.write().map_err(|e| eyre!("{:?}", &e))?;
            Ok(item.error.take())
        } else {
            Ok(None)
        }
    }
}
