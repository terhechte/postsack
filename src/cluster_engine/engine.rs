use std::ops::{Range, RangeInclusive};

use cached::{Cached, SizedCache};
use eframe::egui::Rect;
use eyre::{bail, eyre, Result};

use crate::cluster_engine::link::Response;
use crate::database::query::{Field, Filter, Query, ValueField};
use crate::database::query_result::QueryRow;
use crate::types::Config;

use super::link::Link;
use super::partitions::{Partition, Partitions};

// FIXME: Try with lifetimes. For this use case it might just work
pub struct Grouping {
    value: Option<ValueField>,
    field: Field,
    index: usize,
}

impl Grouping {
    pub fn value(&self) -> Option<String> {
        self.value.as_ref().map(|e| e.value().to_string())
    }

    pub fn name(&self) -> &str {
        self.field.as_str()
    }

    pub fn index(&self, in_fields: &[Field]) -> Option<usize> {
        in_fields.iter().position(|p| p == &self.field)
    }
}

// FIXME:!
// - improve the naming: Grouping, Partitions, Partition, Mails(-> Details), ...
//   items_with_size, current_element_count
// - fix "Action".
// - find a way to merge action, query and response in a type-safe manner...
// - rename cluster_engine to model?
// - move the different operations in modules and just share the state
// - replace row_cache with the LRU crate I have open

/// This signifies the action we're currently evaluating
/// It is used for sending requests and receiving responses
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    /// Recalculate the current partition based on a changed grouping
    Recalculate,
    /// Select a new partition
    Select,
    /// Load the mails for the current partition
    Mails,
}

enum LoadingState {
    Loaded(QueryRow),
    Loading,
}

pub struct Engine {
    search_stack: Vec<ValueField>,
    group_by_stack: Vec<Field>,
    link: Link<Action>,
    partitions: Vec<Partitions>,
    /// This is a very simple cache from ranges to rows.
    /// It doesn't account for overlapping ranges.
    /// There's a lot of room for improvement here.
    row_cache: SizedCache<usize, LoadingState>,
}

impl Engine {
    pub fn new(config: &Config) -> Result<Self> {
        let link = super::link::run(&config)?;
        let engine = Engine {
            link,
            search_stack: Vec::new(),
            group_by_stack: vec![default_group_by_stack(0)],
            partitions: Vec::new(),
            row_cache: SizedCache::with_size(10000),
        };
        Ok(engine)
    }

    pub fn start(&mut self) -> Result<()> {
        Ok(self.update((self.make_group_query()?, Action::Select))?)
    }

    pub fn items_with_size(&mut self, rect: Rect) -> Option<&[Partition]> {
        let partition = self.partitions.last_mut()?;
        partition.update_layout(rect);
        Some(partition.items())
    }

    /// The total amount of elements in all the partitions
    pub fn current_element_count(&self) -> usize {
        let partitions = match self.partitions.last() {
            Some(n) => n,
            None => return 0,
        };
        partitions.element_count()
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

    pub fn update_grouping(&mut self, grouping: &Grouping, field: &Field) -> Result<()> {
        self.group_by_stack
            .get_mut(grouping.index)
            .map(|e| *e = field.clone());
        // Remove any rows that were cached for this partition
        self.row_cache.cache_clear();
        self.update((self.make_group_query()?, Action::Recalculate))
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
        self.update((self.make_group_query()?, Action::Select))
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

        // Remove any rows that were cached for this partition
        self.row_cache.cache_clear();
    }

    // Send the last action over the wire to be calculated
    fn update(&mut self, payload: (Query, Action)) -> Result<()> {
        Ok(self.link.request(&payload.0, payload.1)?)
    }

    /// Fetch the channels to see if there're any updates
    pub fn process(&mut self) -> Result<()> {
        let response = match self.link.receive()? {
            Some(n) => n,
            None => return Ok(()),
        };

        match response {
            Response::Grouped(_, Action::Select, p) => {
                self.partitions.push(p);
                // Remove any rows that were cached for this partition
                self.row_cache.cache_clear();
            }
            Response::Grouped(_, Action::Recalculate, p) => {
                let len = self.partitions.len();
                self.partitions[len - 1] = p;
                // Remove any rows that were cached for this partition
                self.row_cache.cache_clear();
            }
            Response::Normal(Query::Normal { range, .. }, Action::Mails, r) => {
                for (index, row) in range.zip(r) {
                    let entry = LoadingState::Loaded(row.clone());
                    self.row_cache.cache_set(index, entry);
                }
            }
            _ => bail!("Invalid Query / Response combination"),
        }

        Ok(())
    }

    /// Return all group fields which are still available based
    /// on the current stack.
    /// Also always include the current one, so we can choose between
    pub fn available_group_by_fields(&self, grouping: &Grouping) -> Vec<Field> {
        Field::all_cases()
            .filter_map(|f| {
                if f == grouping.field {
                    return Some(f.clone());
                }
                if self.group_by_stack.contains(&f) {
                    None
                } else {
                    Some(f.clone())
                }
            })
            .collect()
    }

    /// Query the contents for the current filter settings.
    /// This call will return the available data and request additional data when it is missing.
    /// The return value indicates whether a row is loaded or loading.
    pub fn current_contents(&mut self, range: &Range<usize>) -> Result<Vec<Option<QueryRow>>> {
        // build an array with either empty values or values from our cache.
        let mut rows = Vec::new();

        let mut missing_data = false;
        for index in range.clone() {
            let entry = self.row_cache.cache_get(&index);
            let entry = match entry {
                Some(LoadingState::Loaded(n)) => Some((*n).clone()),
                Some(LoadingState::Loading) => None,
                None => {
                    // for simplicity, we keep the "something is missing" state separate
                    missing_data = true;

                    // Mark the row as being loaded
                    self.row_cache.cache_set(index, LoadingState::Loading);
                    None
                }
            };
            rows.push(entry);
        }
        // Only if at least some data is missing do we perform the request
        if missing_data && !range.is_empty() {
            let request = self.make_normal_query(range.clone());
            self.link.request(&request, Action::Mails)?;
        }
        Ok(rows)
    }

    pub fn is_busy(&self) -> bool {
        self.is_partitions_busy() || self.is_mail_busy()
    }

    /// When we don't have partitions loaded yet, or
    /// when we're currently querying / loading new partitions
    pub fn is_partitions_busy(&self) -> bool {
        self.partitions.is_empty()
    }

    /// If we're loading mails
    pub fn is_mail_busy(&self) -> bool {
        self.link.is_processing()
    }

    fn make_group_query(&self) -> Result<Query> {
        let mut filters = Vec::new();
        for entry in &self.search_stack {
            filters.push(Filter::Like(entry.clone()));
        }
        let last = self
            .group_by_stack
            .last()
            .ok_or(eyre!("Invalid partition state"))?;
        Ok(Query::Grouped {
            filters,
            group_by: last.clone(),
        })
    }

    fn make_normal_query(&self, range: Range<usize>) -> Query {
        let mut filters = Vec::new();
        for entry in &self.search_stack {
            filters.push(Filter::Like(entry.clone()));
        }
        Query::Normal {
            filters,
            fields: vec![Field::SenderDomain, Field::SenderLocalPart, Field::Subject],
            range,
        }
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
