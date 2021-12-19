//! Operations on `Segmentations`
//!
//! `Segmentations` are collections of `Segments` based on an aggregation of `Items`.
//!
//! A `Segmentation` can be changed to be aggregated on a different `Field`.
//!
//! - [`crate::model::segmentations::aggregation_fields`]
//! - [`crate::model::segmentations::aggregated_by`]
//! - [`crate::model::segmentations::set_aggregation`]
//!
//! A `Segmentation` can be changed to only return a `Range` of segments.
//!
//! - [`crate::model::segmentations::segments_range`]
//! - [`crate::model::segmentations::set_segments_range`]
//!
//! A [`crate::model::Segmentation`] has multiple [`crate::model::Segment`]s which each can be layouted
//! to fit into a rectangle.
//!
//! - [`crate::model::segmentations::layouted_segments]

use eyre::{eyre, Result};

use super::engine::Action;
use super::{
    types::{self, Aggregation, Segment},
    Engine,
};
use crate::database::query::{Field, Filter, Query};
use std::ops::RangeInclusive;

/// Filter the `Range` of segments of the current `Segmentation`
///
/// Returns the `Range` and the total number of segments.
/// If no custom range has been set with [`set_segments_range`], returns
/// the full range of items, otherwise the custom range.
///
/// Returns `None` if no current `Segmentation` exists.
///
/// # Arguments
///
/// * `engine` - The engine to use for retrieving data
/// * `aggregation` - The aggregation to return the fields for. Required to also return the current aggregation field.
pub fn segments_range(engine: &Engine) -> Option<(RangeInclusive<usize>, usize)> {
    let segmentation = engine.segmentations.last()?;
    let len = segmentation.len();
    Some(match &segmentation.range {
        Some(n) => (0..=len, *n.end()),
        None => (0..=len, len),
    })
}

/// Set the `Range` of segments of the current `Segmentation`
///
/// # Arguments
///
/// * `engine` - The engine to use for setting data
/// * `range` - The range to apply. `None` to reset it to all `Segments`
pub fn set_segments_range(engine: &mut Engine, range: Option<RangeInclusive<usize>>) {
    if let Some(n) = engine.segmentations.last_mut() {
        // Make sure the range does not go beyond the current semgents count
        if let Some(r) = range {
            let len = n.len();
            if len > *r.start() && *r.end() < len {
                n.range = Some(r);
            }
        } else {
            n.range = None;
        }
    }
}

/// Additional filters to use in the query
///
/// These filters will be evaluated in addition to the `segmentation` conditions
/// in the query.
/// Setting this value will recalculate the current segmentations.
pub fn set_filters(engine: &mut Engine, filters: &[Filter]) -> Result<()> {
    engine.filters = filters.to_vec();

    // Remove any rows that were cached for this Segmentation
    engine.item_cache.clear();
    engine
        .link
        .request(&make_query(engine)?, Action::RecalculateSegmentation)
}

/// The fields available for the given aggregation
///
/// As the user `pushes` Segmentations and dives into the data,
/// less fields become available to aggregate by. It is inconsequential
/// to aggregate, say, by year, then by month, and then again by year.
/// This method returns the possible fields still available for aggregation.
///
/// # Arguments
///
/// * `engine` - The engine to use for retrieving data
/// * `aggregation` - The aggregation to return the fields for. Required to also return the current aggregation field.
pub fn aggregation_fields(engine: &Engine, aggregation: &Aggregation) -> Vec<Field> {
    #[allow(clippy::unnecessary_filter_map)]
    Field::all_cases()
        .filter_map(|f| {
            if f == aggregation.field {
                return Some(f);
            }
            if engine.group_by_stack.contains(&f) {
                None
            } else {
                Some(f)
            }
        })
        .collect()
}

/// Return all `Aggregation`s applied for the current `Segmentation`
///
/// E.g. if we're first aggregating by Year, and then by Month, this
/// will return a `Vec` of `[Year, Month]`.
///
/// # Arguments
///
/// * `engine` - The engine to use for retrieving data
pub fn aggregated_by(engine: &Engine) -> Vec<Aggregation> {
    let mut result = Vec::new();
    // for everything in the current stack
    let len = engine.group_by_stack.len();
    for (index, field) in engine.group_by_stack.iter().enumerate() {
        let value = match (
            len,
            engine.segmentations.get(index).map(|e| e.selected.as_ref()),
        ) {
            (n, Some(Some(segment))) if len == n => Some(segment.field.clone()),
            _ => None,
        };
        result.push(Aggregation {
            value,
            field: *field,
            index,
        });
    }
    result
}

/// Change the `Field` in the given `Aggregation` to the new one.
///
/// The `Aggregation` will identify the `Segmentation` to use. So this function
/// can be used to change the way a `Segmentation` is the aggregated.
///
/// Retrieve the available aggregations with [`crate::model::segmentations::aggregated_by`].
///
/// # Arguments
///
/// * `engine` - The engine to use for retrieving data
/// * `aggregation` - The aggregation to change
/// * `field` - The field to aggregate the `aggregation` by.
pub fn set_aggregation(
    engine: &mut Engine,
    aggregation: &Aggregation,
    field: &Field,
) -> Result<()> {
    if let Some(e) = engine.group_by_stack.get_mut(aggregation.index) {
        *e = *field;
    }
    // Remove any rows that were cached for this Segmentation
    engine.item_cache.clear();
    engine
        .link
        .request(&make_query(engine)?, Action::RecalculateSegmentation)
}

/// Return the `Segment`s in the current `Segmentation`. Apply layout based on `Rect`.
///
/// It will perform the calculations so that all segments fit into bounds.
/// The results will be applied to each `Segment`.
///
/// Returns the layouted segments.
///
/// # Arguments
///
/// * `engine` - The engine to use for retrieving data
/// * `Rect` - The bounds into which the segments have to fit.
pub fn layouted_segments(engine: &mut Engine, bounds: types::Rect) -> Option<&[Segment]> {
    let segmentation = engine.segmentations.last_mut()?;
    segmentation.update_layout(bounds);
    Some(segmentation.items())
}

/// Can another level of aggregation be performed?
pub fn can_aggregate_more(engine: &Engine) -> bool {
    let index = engine.group_by_stack.len();
    super::engine::default_group_by_stack(index).is_some()
}

/// Perform the query that returns an aggregated `Segmentation`
pub(super) fn make_query(engine: &Engine) -> Result<Query> {
    let mut filters = Vec::new();
    for entry in &engine.search_stack {
        filters.push(Filter::Like(entry.clone()));
    }
    for entry in &engine.filters {
        filters.push(entry.clone());
    }
    let last = engine
        .group_by_stack
        .last()
        .ok_or_else(|| eyre!("Invalid Segmentation state"))?;
    Ok(Query::Grouped {
        filters,
        group_by: *last,
    })
}
