// FIXME: Have our own rect type in here which is compatible to
// treemap and egui
use eframe::egui::Rect;
use eyre::Result;

use crate::database::query::{Filter, GroupByField, ValueField};
use crate::types::Config;

use super::calc::{Link, Request};
use super::partitions::{Partition, Partitions};
use super::IntoRequest;

// FIXME: Use strum or one of the enum to string crates
const DEFAULT_GROUP_BY_FIELDS: &[GroupByField] = {
    use GroupByField::*;
    &[
        SenderDomain,
        SenderLocalPart,
        SenderName,
        Year,
        Month,
        Day,
        ToGroup,
        ToName,
        ToAddress,
        IsReply,
        IsSend,
    ]
};

pub struct Engine {
    search_stack: Vec<ValueField>,
    group_by_stack: Vec<usize>,
    link: Link,
    partitions: Vec<Partitions>,
    // Are we currently waiting for SQLite to finish loading data
    is_querying: bool,
    // Should we recalculate because something in the state changed?
    // This is evaluated during `process` which should be called each `update` loop
    should_recalculate: bool,
}

impl Engine {
    pub fn new(config: &Config) -> Result<Self> {
        let link = super::calc::run(&config)?;
        let engine = Engine {
            link,
            search_stack: Vec::new(),
            group_by_stack: vec![default_group_by_stack(0)],
            partitions: Vec::new(),
            is_querying: false,
            should_recalculate: false,
        };
        Ok(engine)
    }

    pub fn items_with_size(&mut self, rect: Rect) -> Option<&[Partition]> {
        self.partitions.last_mut()?.update_layout(rect);
        Some(self.partitions.last().as_ref()?.items.as_slice())
    }

    /// Returns (index in the `group_by_stack`, index of the chosen group, value of the group if selected)
    pub fn current_groupings(&self) -> Vec<(usize, usize, Option<ValueField>)> {
        let mut result = Vec::new();
        // for everything in the current stack
        for (index, stack_index) in self.group_by_stack.iter().enumerate() {
            let value = if let Some(Some(partition)) =
                self.partitions.get(index).map(|e| e.selected.as_ref())
            {
                Some(partition.field.clone())
            } else {
                None
            };
            result.push((index, *stack_index, value));
        }
        result
    }

    pub fn update_grouping(&mut self, index: usize, group_index: usize) {
        self.group_by_stack.get_mut(index).map(|e| *e = group_index);
        self.should_recalculate = true;
    }

    pub fn select_partition<S: IntoRequest>(
        &mut self,
        partition: Partition,
        state: &S,
    ) -> Result<()> {
        // Assign the partition
        let current = match self.partitions.last_mut() {
            Some(n) => n,
            None => return Ok(()),
        };
        current.selected = Some(partition);

        // Create the new search stack
        self.search_stack = self
            .partitions
            .iter()
            .filter_map(|e| e.selected.as_ref())
            .map(|p| p.field.clone())
            .collect();

        // Add the next group by
        let index = self.group_by_stack.len();
        let next = default_group_by_stack(index);
        self.group_by_stack.push(next);

        // Block UI & Wait for updates
        self.is_querying = true;
        self.update(state)
    }

    pub fn back(&mut self) {
        // FIXME: Checks
        self.group_by_stack.remove(self.group_by_stack.len() - 1);
        self.partitions.remove(self.partitions.len() - 1);
        self.search_stack.remove(self.search_stack.len() - 1);
    }

    pub fn update<S: IntoRequest>(&mut self, state: &S) -> Result<()> {
        self.link.input_sender.send(self.request_from(state))?;
        self.is_querying = true;
        Ok(())
    }

    /// Fetch the channels to see if there're any updates
    pub fn process<S: IntoRequest>(&mut self, state: &S) -> Result<()> {
        match self.link.output_receiver.try_recv() {
            // We received something
            Ok(Ok(p)) => {
                self.partitions.push(p);
                self.is_querying = false;
            }
            // We received nothing
            Err(_) => (),
            // There was an error, we forward it
            Ok(Err(e)) => return Err(e),
        };
        if self.should_recalculate {
            self.should_recalculate = false;
            self.update(state)
        } else {
            Ok(())
        }
    }

    /// When we don't have partitions loaded yet, or
    /// when we're currently querying / loading new partitions
    pub fn is_busy(&self) -> bool {
        self.partitions.is_empty() || self.is_querying || self.should_recalculate
    }

    fn request_from<S: IntoRequest>(&self, state: &S) -> Request {
        let mut filters = state.into_filters();

        // For each assigned partition, we use the term and value as an addition search
        for field in &self.search_stack {
            filters.push(Filter::Is(field.clone()));
        }

        Request {
            filters,
            fields: self.group_by_fields(),
        }
    }
}

// FIXME: Try to get rid of these
impl Engine {
    pub fn group_by_field_for(index: usize) -> GroupByField {
        DEFAULT_GROUP_BY_FIELDS[index]
    }

    pub fn all_group_by_fields() -> Vec<GroupByField> {
        DEFAULT_GROUP_BY_FIELDS.into()
    }

    pub fn group_by_fields(&self) -> Vec<GroupByField> {
        self.group_by_stack
            .iter()
            .map(|e| Self::group_by_field_for(*e))
            .collect()
    }
}

/// Return the default group by fields index for each stack entry
pub fn default_group_by_stack(index: usize) -> usize {
    let f = match index {
        0 => GroupByField::Year,
        1 => GroupByField::SenderDomain,
        2 => GroupByField::SenderLocalPart,
        3 => GroupByField::Month,
        4 => GroupByField::Day,
        _ => panic!(),
    };
    DEFAULT_GROUP_BY_FIELDS
        .iter()
        .position(|e| e == &f)
        .unwrap()
}
