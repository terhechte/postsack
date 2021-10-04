use std::ops::RangeInclusive;

use eframe::egui::Rect;
use eyre::{eyre, Result};

use crate::database::query::{Filter, GroupByField, ValueField};
use crate::types::Config;

use super::calc::{Link, Request};
use super::partitions::{Partition, Partitions};

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

// FIXME: Try with lifetimes. For this use case it might just work
pub struct Grouping {
    value: Option<ValueField>,
    field: GroupByField,
    index: usize,
}

impl Grouping {
    pub fn value(&self) -> Option<String> {
        self.value.as_ref().map(|e| e.value().to_string())
    }

    pub fn name(&self) -> &str {
        self.field.as_str()
    }

    pub fn index(&self, in_fields: &[GroupByField]) -> Option<usize> {
        in_fields.iter().position(|p| p == &self.field)
    }
}

/// This signifies the action we're currently evaluating
/// It is used for sending requests and receiving responses
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    Recalculate,
    Select,
    Wait,
}

pub struct Engine {
    search_stack: Vec<ValueField>,
    group_by_stack: Vec<GroupByField>,
    link: Link<Action>,
    partitions: Vec<Partitions>,
    action: Option<Action>,
}

impl Engine {
    pub fn new(config: &Config) -> Result<Self> {
        let link = super::calc::run(&config)?;
        let engine = Engine {
            link,
            search_stack: Vec::new(),
            group_by_stack: vec![default_group_by_stack(0)],
            partitions: Vec::new(),
            action: None,
        };
        Ok(engine)
    }

    pub fn start(&mut self) -> Result<()> {
        // Make the initial query
        self.action = Some(Action::Select);
        self.update()
    }

    pub fn items_with_size(&mut self, rect: Rect) -> Option<&[Partition]> {
        let partition = self.partitions.last_mut()?;
        partition.update_layout(rect);
        Some(partition.items())
    }

    /// Retrieve the min and max amount of items. The range that should be displayed.
    /// Per default, it is the whole range of the partition
    pub fn current_range(&self) -> Option<(RangeInclusive<usize>, usize)> {
        let partition = self.partitions.last()?;
        let len = partition.len();
        let r = match &partition.range {
            Some(n) => (0..=len, *n.end()),
            None => (0..=len, len),
        };
        Some(r)
    }

    pub fn set_current_range(&mut self, range: Option<RangeInclusive<usize>>) -> Option<()> {
        match self.partitions.last_mut() {
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
    pub fn current_groupings(&self) -> Vec<Grouping> {
        let mut result = Vec::new();
        // for everything in the current stack
        let len = self.group_by_stack.len();
        for (index, field) in self.group_by_stack.iter().enumerate() {
            let value = match (len, self.partitions.get(index).map(|e| e.selected.as_ref())) {
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

    pub fn update_grouping(&mut self, grouping: &Grouping, field: &GroupByField) -> Result<()> {
        self.group_by_stack
            .get_mut(grouping.index)
            .map(|e| *e = field.clone());
        self.action = Some(Action::Recalculate);
        self.update()
    }

    pub fn select_partition(&mut self, partition: Partition) -> Result<()> {
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
        self.action = Some(Action::Select);
        self.update()
    }

    pub fn back(&mut self) {
        if self.group_by_stack.is_empty()
            || self.partitions.is_empty()
            || self.search_stack.is_empty()
        {
            tracing::error!(
                "Invalid state. Not everything has the same length: {:?}, {:?}, {:?}",
                &self.group_by_stack,
                self.partitions,
                self.search_stack
            );
            return;
        }

        // Remove the last entry of everything
        self.group_by_stack.remove(self.group_by_stack.len() - 1);
        self.partitions.remove(self.partitions.len() - 1);
        self.search_stack.remove(self.search_stack.len() - 1);

        // Remove the selection in the last partition
        self.partitions.last_mut().map(|e| e.selected = None);
    }

    // Send the last action over the wire to be calculated
    fn update(&mut self) -> Result<()> {
        let action = match self.action {
            Some(n) => n,
            None => return Ok(()),
        };
        let request = self.make_group_request().ok_or(eyre!("Invalid State."))?;
        self.link.input_sender.send((request, action))?;
        self.action = Some(Action::Wait);
        Ok(())
    }

    /// Fetch the channels to see if there're any updates
    pub fn process(&mut self) -> Result<()> {
        match self.link.output_receiver.try_recv() {
            // We received something
            Ok(Ok((p, action))) => {
                match action {
                    Action::Select => self.partitions.push(p),
                    Action::Recalculate => {
                        let len = self.partitions.len();
                        self.partitions[len - 1] = p;
                    }
                    Action::Wait => panic!("Should never send a wait action into the other thread"),
                }
                self.action = None;
            }
            // We received nothing
            Err(_) => (),
            // There was an error, we forward it
            Ok(Err(e)) => return Err(e),
        };
        Ok(())
    }

    /// Return all group fields which are still available based
    /// on the current stack.
    /// Also always include the current one, so we can choose between
    pub fn available_group_by_fields(&self, grouping: &Grouping) -> Vec<GroupByField> {
        DEFAULT_GROUP_BY_FIELDS
            .iter()
            .filter_map(|f| {
                if f == &grouping.field {
                    return Some(f.clone());
                }
                if self.group_by_stack.contains(f) {
                    None
                } else {
                    Some(f.clone())
                }
            })
            .collect()
    }

    /// Query the contents for the current filter settings
    /// This is a blocking call to simplify things a great deal
    pub fn current_contents(&mut self, range: std::ops::Range<usize>) -> Result<Vec<ValueField>> {
        todo!()
    }

    /// When we don't have partitions loaded yet, or
    /// when we're currently querying / loading new partitions
    pub fn is_busy(&self) -> bool {
        self.partitions.is_empty() || self.action.is_some()
    }

    fn make_group_request(&self) -> Option<Request> {
        let mut filters = Vec::new();
        for entry in &self.search_stack {
            filters.push(Filter::Like(entry.clone()));
        }
        Some(Request::Grouped {
            filters,
            group_by: self.group_by_stack.last()?.clone(),
        })
    }
}

/// Return the default group by fields index for each stack entry
pub fn default_group_by_stack(index: usize) -> GroupByField {
    match index {
        0 => GroupByField::Year,
        1 => GroupByField::SenderDomain,
        2 => GroupByField::SenderLocalPart,
        3 => GroupByField::Month,
        4 => GroupByField::Day,
        _ => panic!(),
    }
}
