use std::convert::TryFrom;

use eframe::egui::Rect as EguiRect;
use eyre::{Report, Result};
use treemap::{Mappable, Rect};

use crate::database::{query::ValueField, query_result::QueryResult};

#[derive(Debug, Clone)]
pub struct Segment {
    pub field: ValueField,
    pub count: usize,
    /// A TreeMap Rect
    pub rect: Rect,
}

impl Segment {
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

impl Mappable for Segment {
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

impl<'a> TryFrom<&'a QueryResult> for Segment {
    type Error = Report;
    fn try_from(result: &'a QueryResult) -> Result<Self> {
        let (count, field) = match result {
            QueryResult::Grouped { count, value } => (count, value),
            _ => return Err(eyre::eyre!("Invalid result type, expected `Grouped`")),
        };
        // so far we can only support one group by at a time.
        // at least in here. The queries support it

        Ok(Segment {
            field: field.clone(),
            count: *count,
            rect: Rect::new(),
        })
    }
}
