use treemap::{self, TreemapLayout};

use super::segment::Segment;
use super::Rect;

/// A small NewType so that we can keep all the `TreeMap` code in here and don't
/// have to do the layout calculation in a widget.
#[derive(Debug)]
pub struct Segmentation {
    items: Vec<Segment>,
    pub selected: Option<Segment>,
    pub range: Option<std::ops::RangeInclusive<usize>>,
}

impl Segmentation {
    pub fn new(items: Vec<Segment>) -> Self {
        Self {
            items,
            selected: None,
            range: None,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Update the layout information in the Segments
    /// based on the current size
    pub fn update_layout(&mut self, rect: Rect) {
        let layout = TreemapLayout::new();
        let bounds = treemap::Rect::from_points(rect.left, rect.top, rect.width, rect.height);
        layout.layout_items(self.items(), bounds);
    }

    /// The total amount of items in all the `Segments`.
    /// E.g. the sum of the count of the `Segments`
    pub fn element_count(&self) -> usize {
        self.items.iter().map(|e| e.count).sum::<usize>()
    }

    /// The items in this `Segmentation`, with range applied
    pub fn items(&mut self) -> &mut [Segment] {
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
