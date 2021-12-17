use std::path::Path;
use std::thread::JoinHandle;

use crossbeam_channel::Sender;
use eyre::Result;

use crate::Config;

use super::{db_message::DBMessage, query::Query, query_result::QueryResult};

pub trait DatabaseQuery: Send + 'static {
    fn query(&self, query: &Query) -> Result<Vec<QueryResult>>;
}

pub trait DatabaseLike: DatabaseQuery + Clone {
    fn new(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
    fn config(path: impl AsRef<Path>) -> Result<Config>
    where
        Self: Sized;
    fn total_mails(&self) -> Result<usize>;
    fn import(self) -> (Sender<DBMessage>, JoinHandle<Result<usize>>);
    fn save_config(&self, config: Config) -> Result<()>;
}
