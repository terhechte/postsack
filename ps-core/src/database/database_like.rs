use eyre::Result;
use std::path::Path;

use super::{query::Query, query_result::QueryResult};

pub trait DatabaseLike: Send + Sync {
    fn new(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
    fn total_mails(&self) -> Result<usize>;
    fn query(&self, query: &Query) -> Result<Vec<QueryResult>>;
}
