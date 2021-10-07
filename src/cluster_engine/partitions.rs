//! Operations on the currently visible **Partition**.

use cached::Cached;
use eyre::{eyre, Result};

use super::engine::Action;
use super::{
    types::{Grouping, Partition},
    Engine,
};
use crate::database::query::{Field, Filter, Query};
use std::ops::RangeInclusive;

/// Return all group fields which are still available based
/// on the current stack.
/// Also always include the current one, so we can choose between
pub fn available_group_by_fields(engine: &Engine, grouping: &Grouping) -> Vec<Field> {
    Field::all_cases()
        .filter_map(|f| {
            if f == grouping.field {
                return Some(f.clone());
            }
            if engine.group_by_stack.contains(&f) {
                None
            } else {
                Some(f.clone())
            }
        })
        .collect()
}

/// Retrieve the min and max amount of items. The range that should be displayed.
/// Per default, it is the whole range of the partition
pub fn current_range(engine: &Engine) -> Option<(RangeInclusive<usize>, usize)> {
    let partition = engine.partitions.last()?;
    let len = partition.len();
    let r = match &partition.range {
        Some(n) => (0..=len, *n.end()),
        None => (0..=len, len),
    };
    Some(r)
}

pub fn set_current_range(engine: &mut Engine, range: Option<RangeInclusive<usize>>) -> Option<()> {
    match engine.partitions.last_mut() {
        Some(n) => {
            if let Some(r) = range {
                let len = n.len();
                if len > *r.start() && *r.end() < len {
                    n.range = Some(r.clone());
                    Some(())
                } else {
                    None
                }
            } else {
                n.range = None;
                Some(())
            }
        }
        None => None,
    }
}

/// Returns (index in the `group_by_stack`, index of the chosen group, value of the group if selected)
pub fn current_groupings(engine: &Engine) -> Vec<Grouping> {
    let mut result = Vec::new();
    // for everything in the current stack
    let len = engine.group_by_stack.len();
    for (index, field) in engine.group_by_stack.iter().enumerate() {
        let value = match (
            len,
            engine.partitions.get(index).map(|e| e.selected.as_ref()),
        ) {
            (n, Some(Some(partition))) if len == n => Some(partition.field.clone()),
            _ => None,
        };
        result.push(Grouping {
            value,
            field: field.clone(),
            index,
        });
    }
    result
}

pub fn update_grouping(engine: &mut Engine, grouping: &Grouping, field: &Field) -> Result<()> {
    engine
        .group_by_stack
        .get_mut(grouping.index)
        .map(|e| *e = field.clone());
    // Remove any rows that were cached for this partition
    engine.row_cache.cache_clear();
    engine.update((make_group_query(engine)?, Action::Recalculate))
}

pub fn items_with_size(engine: &mut Engine, rect: eframe::egui::Rect) -> Option<&[Partition]> {
    let partition = engine.partitions.last_mut()?;
    partition.update_layout(rect);
    Some(partition.items())
}

/// When we don't have partitions loaded yet, or
/// when we're currently querying / loading new partitions
pub fn is_partitions_busy(engine: &Engine) -> bool {
    engine.partitions.is_empty()
}

pub(super) fn make_group_query(engine: &Engine) -> Result<Query> {
    let mut filters = Vec::new();
    for entry in &engine.search_stack {
        filters.push(Filter::Like(entry.clone()));
    }
    let last = engine
        .group_by_stack
        .last()
        .ok_or(eyre!("Invalid partition state"))?;
    Ok(Query::Grouped {
        filters,
        group_by: last.clone(),
    })
}
