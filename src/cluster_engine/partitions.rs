use std::convert::TryFrom;

use eframe::egui::Rect as EguiRect;
use eyre::{Report, Result};
use treemap::{Mappable, Rect, TreemapLayout};

use crate::database::{query::ValueField, query_result::QueryResult};

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
pub struct Partitions {
    pub items: Vec<Partition>,
    pub selected: Option<Partition>,
}

impl Partitions {
    pub fn new(items: Vec<Partition>) -> Self {
        Self {
            items,
            selected: None,
        }
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
        layout.layout_items(&mut self.items, bounds);
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
    fn try_from(r: &'a QueryResult) -> Result<Self> {
        // so far we can only support one group by at a time.
        // at least in here. The queries support it
        let field = r
            .values
            .first()
            .ok_or(eyre::eyre!("No group by fields available"))?;

        Ok(Partition {
            field: field.clone(),
            count: r.count,
            rect: Rect::new(),
        })
    }
}