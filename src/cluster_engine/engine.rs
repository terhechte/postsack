//! The `Engine` is the entry point to the data that should be
//! displayed in Segmentations.
//! See [`Engine`] for more information.
//! See also:
//! - [`segmentations::`]
//! - [`items::`]
use cached::{Cached, SizedCache};
use eyre::{bail, Result};

use crate::cluster_engine::link::Response;
use crate::database::query::{Field, Query, ValueField};
use crate::types::Config;

use super::link::Link;
use super::segmentation;
use super::types::{LoadingState, Segment, Segmentation};

/// This signifies the action we're currently evaluating
/// It is used for sending requests and receiving responses
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum Action {
    /// Recalculate the current `Segmentation` based on a changed aggregation
    RecalculateSegmentation,
    /// Push a new `Segmentation`
    PushSegmentation,
    /// Load the mails for the current `Segmentation`
    LoadItems,
}

/// Interact with the `Database`, operate on `Segmentations`, `Segments`, and `Items`.
/// `Engine` is used as the input for almost all operations in the
/// `items::` and `segmentation::` modules.
pub struct Engine {
    pub(super) search_stack: Vec<ValueField>,
    pub(super) group_by_stack: Vec<Field>,
    pub(super) link: Link<Action>,
    pub(super) segmentations: Vec<Segmentation>,
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
            segmentations: Vec::new(),
            item_cache: SizedCache::with_size(10000),
        };
        Ok(engine)
    }

    /// Start the `Engine`. This will create a thread to
    /// asynchronously communicate with the underlying backend
    /// in a non-blocking manner.
    pub fn start(&mut self) -> Result<()> {
        Ok(self
            .link
            .request(&segmentation::make_query(&self)?, Action::PushSegmentation)?)
    }

    /// Return the current stack of `Segmentations`
    pub fn segmentations(&self) -> &[Segmentation] {
        &self.segmentations
    }

    /// Push a new `Segment` to select a more specific `Segmentation`.
    ///
    /// Pushing will create an additional `Aggregation` based on the selected
    /// `Segment`, retrieve the data from the backend, and add it to the
    /// current stack of `Segmentations`.
    /// It allows to **drill down** into the data.
    pub fn push(&mut self, segment: Segment) -> Result<()> {
        // Assign the segmentation
        let current = match self.segmentations.last_mut() {
            Some(n) => n,
            None => return Ok(()),
        };
        current.selected = Some(segment);

        // Create the new search stack
        self.search_stack = self
            .segmentations
            .iter()
            .filter_map(|e| e.selected.as_ref())
            .map(|p| p.field.clone())
            .collect();

        // Add the next group by
        let index = self.group_by_stack.len();
        let next = default_group_by_stack(index);
        self.group_by_stack.push(next);

        // Block UI & Wait for updates
        self.link
            .request(&segmentation::make_query(&self)?, Action::PushSegmentation)
    }

    /// Pop the current `Segmentation` from the stack.
    /// The opposite of [`engine::push`]
    pub fn pop(&mut self) {
        if self.group_by_stack.is_empty()
            || self.segmentations.is_empty()
            || self.search_stack.is_empty()
        {
            tracing::error!(
                "Invalid state. Not everything has the same length: {:?}, {:?}, {:?}",
                &self.group_by_stack,
                self.segmentations,
                self.search_stack
            );
            return;
        }

        // Remove the last entry of everything
        self.group_by_stack.remove(self.group_by_stack.len() - 1);
        self.segmentations.remove(self.segmentations.len() - 1);
        self.search_stack.remove(self.search_stack.len() - 1);

        // Remove the selection in the last segmentation
        self.segmentations.last_mut().map(|e| e.selected = None);

        // Remove any rows that were cached for this segmentation
        self.item_cache.cache_clear();
    }

    /// Call this continously to retrieve calculation results and apply them.
    /// Any mutating function on [`Engine`], such as [`Engine::push`] or [`items::items`]
    /// require calling this method to apply there results once they're
    /// available from the asynchronous backend.
    /// This method is specifically non-blocking for usage in
    /// `Eventloop` based UI frameworks such as `egui`.
    pub fn process(&mut self) -> Result<()> {
        let response = match self.link.receive()? {
            Some(n) => n,
            None => return Ok(()),
        };

        match response {
            Response::Grouped(_, Action::PushSegmentation, p) => {
                self.segmentations.push(p);
                // Remove any rows that were cached for this segmentation
                self.item_cache.cache_clear();
            }
            Response::Grouped(_, Action::RecalculateSegmentation, p) => {
                let len = self.segmentations.len();
                self.segmentations[len - 1] = p;
                // Remove any rows that were cached for this segmentation
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

    /// Returns true if there're currently calculations open and `process`
    /// needs to be called. This can be used in `Eventloop` based frameworks
    /// such as `egui` to know when to continue calling `process` in the `loop`
    /// ```
    /// loop {
    ///  self.engine.process().unwrap();
    ///  if self.engine.is_busy() {
    ///    // Call the library function to run the event-loop again.
    ///    ctx.request_repaint();
    ///  }
    /// }
    /// ```
    pub fn is_busy(&self) -> bool {
        self.link.is_processing() || self.segmentations.is_empty()
    }
}

/// Return the default aggregation fields for each segmentation stack level
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
