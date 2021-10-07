use cached::Cached;
use eyre::Result;

use super::types::LoadingState;
use super::{engine::Action, Engine};
use crate::database::{
    query::{Field, Filter, Query},
    query_result::QueryRow,
};

use std::ops::Range;

/// Query the contents for the current filter settings.
/// This call will return the available data and request additional data when it is missing.
/// The return value indicates whether a row is loaded or loading.
pub fn current_contents(
    engine: &mut Engine,
    range: &Range<usize>,
) -> Result<Vec<Option<QueryRow>>> {
    // build an array with either empty values or values from our cache.
    let mut rows = Vec::new();

    let mut missing_data = false;
    for index in range.clone() {
        let entry = engine.row_cache.cache_get(&index);
        let entry = match entry {
            Some(LoadingState::Loaded(n)) => Some((*n).clone()),
            Some(LoadingState::Loading) => None,
            None => {
                // for simplicity, we keep the "something is missing" state separate
                missing_data = true;

                // Mark the row as being loaded
                engine.row_cache.cache_set(index, LoadingState::Loading);
                None
            }
        };
        rows.push(entry);
    }
    // Only if at least some data is missing do we perform the request
    if missing_data && !range.is_empty() {
        let request = make_normal_query(&engine, range.clone());
        engine.link.request(&request, Action::Mails)?;
    }
    Ok(rows)
}

/// The total amount of elements in all the partitions
pub fn current_element_count(engine: &Engine) -> usize {
    let partitions = match engine.partitions.last() {
        Some(n) => n,
        None => return 0,
    };
    partitions.element_count()
}

/// If we're loading mails
pub fn is_mail_busy(engine: &Engine) -> bool {
    engine.link.is_processing()
}

fn make_normal_query(engine: &Engine, range: Range<usize>) -> Query {
    let mut filters = Vec::new();
    for entry in &engine.search_stack {
        filters.push(Filter::Like(entry.clone()));
    }
    Query::Normal {
        filters,
        fields: vec![Field::SenderDomain, Field::SenderLocalPart, Field::Subject],
        range,
    }
}
