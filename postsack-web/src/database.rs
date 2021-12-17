use std::path::Path;
use std::thread::JoinHandle;

use ps_core::{
    crossbeam_channel::Sender,
    eyre::{bail, Result},
    Config, DBMessage, DatabaseLike, DatabaseQuery, Field, Query, QueryResult, ValueField,
};

pub struct FakeDatabase {}

impl FakeDatabase {
    pub fn total_item_count() -> usize {
        33
    }
}

impl Clone for FakeDatabase {
    fn clone(&self) -> Self {
        FakeDatabase {}
    }
}

impl DatabaseQuery for FakeDatabase {
    fn query(&self, query: &Query) -> Result<Vec<QueryResult>> {
        Ok((0..50)
            .map(|e| QueryResult::Grouped {
                count: e as usize + 30,
                value: ValueField::usize(&Field::Month, e as usize),
            })
            .collect())
    }
}

impl DatabaseLike for FakeDatabase {
    fn new(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(FakeDatabase {})
    }

    fn config(path: impl AsRef<Path>) -> Result<Config>
    where
        Self: Sized,
    {
        bail!("Na")
    }

    fn total_mails(&self) -> Result<usize> {
        Ok(0)
    }

    fn import(self) -> (Sender<DBMessage>, JoinHandle<Result<usize>>) {
        panic!()
    }
    fn save_config(&self, config: Config) -> Result<()> {
        Ok(())
    }
}
