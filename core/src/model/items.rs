//! Operations related to retrieving `items` from the current `Segmentation`
//!
//! A `Segmentation` is a aggregation of items into many `Segments`.
//! These operations allow retreiving the individual items for all
//! segments in the `Segmentation.

use eyre::Result;

use super::types::LoadingState;
use super::{engine::Action, Engine};
use crate::database::{
    query::{Field, Filter, Query},
    query_result::QueryRow,
};

use std::ops::Range;

/// Return the `items` in the current `Segmentation`
///
/// If the items don't exist in the cache, they will be queried
/// asynchronously from the database. The return value distinguishes
/// between `Loaded` and `Loading` items.
///
/// # Arguments
///
/// * `engine` - The engine to use for retrieving data
/// * `range` - The range of items to retrieve. If `None` then all items will be retrieved
pub fn items(engine: &mut Engine, range: Option<Range<usize>>) -> Result<Vec<Option<QueryRow>>> {
    // build an array with either empty values or values from our cache.
    let mut rows = Vec::new();

    // The given range or all items
    let range = range.unwrap_or_else(|| Range {
        start: 0,
        end: count(engine),
    });

    let mut missing_data = false;
    for index in range.clone() {
        let entry = engine.item_cache.get(&index);
        let entry = match entry {
            Some(LoadingState::Loaded(n)) => Some((*n).clone()),
            Some(LoadingState::Loading) => None,
            None => {
                // for simplicity, we keep the "something is missing" state separate
                missing_data = true;

                // Mark the row as being loaded
                engine.item_cache.put(index, LoadingState::Loading);
                None
            }
        };
        rows.push(entry);
    }
    // Only if at least some data is missing do we perform the request
    if missing_data && !range.is_empty() {
        let request = make_query(engine, range);
        engine.link.request(&request, Action::LoadItems)?;
    }
    Ok(rows)
}

/// The total amount of elements in the current `Segmentation`
///
/// # Arguments
///
/// * `engine` - The engine to use for retrieving data
pub fn count(engine: &Engine) -> usize {
    let segmentation = match engine.segmentations.last() {
        Some(n) => n,
        None => return 0,
    };
    segmentation.element_count()
}

/// Make the query for retrieving items
fn make_query(engine: &Engine, range: Range<usize>) -> Query {
    let mut filters = Vec::new();
    for entry in &engine.search_stack {
        filters.push(Filter::Like(entry.clone()));
    }
    Query::Normal {
        filters,
        fields: vec![
            Field::SenderDomain,
            Field::SenderLocalPart,
            Field::Subject,
            Field::Path,
            Field::Timestamp,
        ],
        range,
    }
}
