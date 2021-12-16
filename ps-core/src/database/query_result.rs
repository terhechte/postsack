use super::query::{Field, ValueField};
use std::collections::HashMap;

pub type QueryRow = HashMap<Field, ValueField>;

#[derive(Debug)]
pub enum QueryResult {
    Grouped {
        /// How many items did we find?
        count: usize,
        /// All the itmes that we grouped by including their values.
        /// So that we can use each of them to limit the next query.
        value: ValueField,
    },
    Normal(QueryRow),
    Other(ValueField),
}
