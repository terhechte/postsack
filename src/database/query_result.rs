use super::query::ValueField;

#[derive(Debug)]
pub struct QueryResult<'a> {
    /// How many items did we find?
    pub count: usize,
    /// All the itmes that we grouped by including their values.
    /// So that we can use each of them to limit the next query.
    pub values: Vec<ValueField<'a>>,
}
