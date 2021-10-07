use std::convert::TryFrom;

use eframe::egui::Rect as EguiRect;
use eyre::{Report, Result};
use treemap::{Mappable, Rect, TreemapLayout};

use crate::database::{
    query::{Field, ValueField},
    query_result::{QueryResult, QueryRow},
};

/// Is a individual row/item being loaded or already loaded.
/// Used in a cache to improve the loading of data for the UI.
pub enum LoadingState {
    Loaded(QueryRow),
    Loading,
}

// FIXME: Try with lifetimes. For this use case it might just work
pub struct Grouping {
    pub(super) value: Option<ValueField>,
    pub(super) field: Field,
    pub(super) index: usize,
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

#[derive(Debug, Clone)]
pub struct Partition {
    pub field: ValueField,
    pub count: usize,
    /// A TreeMap Rect
    pub rect: Rect,
}

impl Partition {
    /// Perform rect conversion from TreeMap to Egui
    pub fn layout_rect(&self) -> EguiRect {
        use eframe::egui::pos2;
        EguiRect {
            min: pos2(self.rect.x as f32, self.rect.y as f32),
            max: pos2(
                self.rect.x as f32 + self.rect.w as f32,
                self.rect.y as f32 + self.rect.h as f32,
            ),
        }
    }
}

/// A small NewType so that we can keep all the `TreeMap` code in here and don't
/// have to do the layout calculation in a widget.
#[derive(Debug)]
pub struct Partitions {
    items: Vec<Partition>,
    pub selected: Option<Partition>,
    pub range: Option<std::ops::RangeInclusive<usize>>,
}

impl Partitions {
    pub fn new(items: Vec<Partition>) -> Self {
        Self {
            items,
            selected: None,
            range: None,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Update the layout information in the partitions
    /// based on the current size
    pub fn update_layout(&mut self, rect: EguiRect) {
        let layout = TreemapLayout::new();
        let bounds = Rect::from_points(
            rect.left() as f64,
            rect.top() as f64,
            rect.width() as f64,
            rect.height() as f64,
        );
        layout.layout_items(&mut self.items(), bounds);
    }

    /// The total amount of items in all the partitions.
    /// E.g. the sum of the count of the partitions
    pub fn element_count(&self) -> usize {
        self.items.iter().map(|e| e.count).sum::<usize>()
    }

    /// The items in this partition, with range applied
    pub fn items(&mut self) -> &mut [Partition] {
        match &self.range {
            Some(n) => {
                // we reverse the range
                let reversed_range = (self.len() - n.end())..=(self.len() - 1);
                &mut self.items[reversed_range]
            }
            None => self.items.as_mut_slice(),
        }
    }
}

impl Mappable for Partition {
    fn size(&self) -> f64 {
        self.count as f64
    }

    fn bounds(&self) -> &Rect {
        &self.rect
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.rect = bounds;
    }
}

impl<'a> TryFrom<&'a QueryResult> for Partition {
    type Error = Report;
    fn try_from(result: &'a QueryResult) -> Result<Self> {
        let (count, field) = match result {
            QueryResult::Grouped { count, value } => (count, value),
            _ => return Err(eyre::eyre!("Invalid result type, expected `Grouped`")),
        };
        // so far we can only support one group by at a time.
        // at least in here. The queries support it

        Ok(Partition {
            field: field.clone(),
            count: *count,
            rect: Rect::new(),
        })
    }
}
