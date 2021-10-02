// FIXME: Have our own rect type in here which is compatible to
// treemap and egui
use eframe::egui::Rect;
use eyre::{bail, Result};

use crate::database::query::{Filter, GroupByField, ValueField};
use crate::types::Config;

use super::calc::{Link, Request};
use super::partitions::{Partition, Partitions};
use super::IntoRequest;

pub type GroupByFieldIndex = usize;

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
    next_partition: Option<Partition>,
    is_querying: bool,
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
            next_partition: None,
            is_querying: false,
            should_recalculate: false,
        };
        Ok(engine)
    }

    pub fn items_with_size(&mut self, rect: Rect) -> Option<&[Partition]> {
        self.partitions.last_mut()?.update_layout(rect);
        Some(self.partitions.last().as_ref()?.items.as_slice())
    }

    pub fn select_partition<S: IntoRequest>(&mut self, partition: Partition, state: &S) {
        // Assign the partition
        let current = match self.partitions.last_mut() {
            Some(n) => n,
            None => return,
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
        self.update(state);
    }

    pub fn back(&mut self) {
        // FIXME: Checks
        self.group_by_stack.remove(self.group_by_stack.len() - 1);
        self.partitions.remove(self.partitions.len() - 1);
        self.search_stack.remove(self.search_stack.len() - 1);
    }

    pub fn update<S: IntoRequest>(&mut self, state: &S) {
        // Submit it
        self.link.input_sender.send(self.request_from(state));
        self.is_querying = true;
    }

    /// Fetch the channels to see if there're any updates
    pub fn process(&mut self) -> Result<()> {
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
        Ok(())
    }

    /// When we don't have partitions loaded yet, or
    /// when we're currently querying / loading new partitions
    pub fn is_busy(&self) -> bool {
        self.partitions.is_empty() || self.is_querying
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
