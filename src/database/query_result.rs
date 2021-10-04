use super::query::ValueField;

#[derive(Debug)]
pub enum QueryResult {
    Grouped {
        /// How many items did we find?
        count: usize,
        /// All the itmes that we grouped by including their values.
        /// So that we can use each of them to limit the next query.
        values: Vec<ValueField>,
    },
    Normal(Vec<ValueField>),
}
