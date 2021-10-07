use cached::{Cached, SizedCache};
use eyre::{bail, Result};

use crate::cluster_engine::link::Response;
use crate::database::query::{Field, Query, ValueField};
use crate::types::Config;

use super::items;
use super::link::Link;
use super::partitions;
use super::types::{LoadingState, Partition, Partitions};

// FIXME:!
// - improve the naming: Grouping, Partitions, Partition, Mails(-> Details), ...
//   items_with_size, current_element_count
// - rename cluster_engine to model?
// - replace row_cache with the LRU crate I have open
// - write method documentation
// - write file header documentation

/// This signifies the action we're currently evaluating
/// It is used for sending requests and receiving responses
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum Action {
    /// Recalculate the current partition based on a changed grouping
    RecalculatePartition,
    /// Select a new partition
    PushPartition,
    /// Load the mails for the current partition
    LoadItems,
}

pub struct Engine {
    pub(super) search_stack: Vec<ValueField>,
    pub(super) group_by_stack: Vec<Field>,
    pub(super) link: Link<Action>,
    pub(super) partitions: Vec<Partitions>,
    /// This is a very simple cache from ranges to rows.
    /// It doesn't account for overlapping ranges.
    /// There's a lot of room for improvement here.
    pub(super) item_cache: SizedCache<usize, LoadingState>,
}

impl Engine {
    pub fn new(config: &Config) -> Result<Self> {
        let link = super::link::run(&config)?;
        let engine = Engine {
            link,
            search_stack: Vec::new(),
            group_by_stack: vec![default_group_by_stack(0)],
            partitions: Vec::new(),
            item_cache: SizedCache::with_size(10000),
        };
        Ok(engine)
    }

    pub fn start(&mut self) -> Result<()> {
        Ok(self.link.request(
            &partitions::make_partition_query(&self)?,
            Action::PushPartition,
        )?)
    }

    pub fn push(&mut self, partition: Partition) -> Result<()> {
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
        self.link.request(
            &partitions::make_partition_query(&self)?,
            Action::PushPartition,
        )
    }

    pub fn pop(&mut self) {
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

        // Remove any rows that were cached for this partition
        self.item_cache.cache_clear();
    }

    /// Fetch the channels to see if there're any updates
    pub fn process(&mut self) -> Result<()> {
        let response = match self.link.receive()? {
            Some(n) => n,
            None => return Ok(()),
        };

        match response {
            Response::Grouped(_, Action::PushPartition, p) => {
                self.partitions.push(p);
                // Remove any rows that were cached for this partition
                self.item_cache.cache_clear();
            }
            Response::Grouped(_, Action::RecalculatePartition, p) => {
                let len = self.partitions.len();
                self.partitions[len - 1] = p;
                // Remove any rows that were cached for this partition
                self.item_cache.cache_clear();
            }
            Response::Normal(Query::Normal { range, .. }, Action::LoadItems, r) => {
                for (index, row) in range.zip(r) {
                    let entry = LoadingState::Loaded(row.clone());
                    self.item_cache.cache_set(index, entry);
                }
            }
            _ => bail!("Invalid Query / Response combination"),
        }

        Ok(())
    }

    pub fn is_busy(&self) -> bool {
        partitions::is_partitions_busy(&self) || items::is_mail_busy(&self)
    }
}

/// Return the default group by fields index for each stack entry
pub fn default_group_by_stack(index: usize) -> Field {
    match index {
        0 => Field::Year,
        1 => Field::SenderDomain,
        2 => Field::SenderLocalPart,
        3 => Field::Month,
        4 => Field::Day,
        _ => panic!(),
    }
}
