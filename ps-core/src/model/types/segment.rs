use super::Rect;
use std::convert::TryFrom;

use eyre::{Report, Result};
use treemap::{self, Mappable};

use crate::database::{query::ValueField, query_result::QueryResult};

#[derive(Debug, Clone)]
pub struct Segment {
    pub field: ValueField,
    pub count: usize,
    /// A TreeMap Rect
    pub rect: treemap::Rect,
}

impl Segment {
    /// Perform rect conversion from TreeMap to the public type
    pub fn layout_rect(&self) -> Rect {
        Rect::new(
            (self.rect.x, self.rect.y),
            (self.rect.x + self.rect.w, self.rect.y + self.rect.h),
        )
    }
}

impl Mappable for Segment {
    fn size(&self) -> f64 {
        self.count as f64
    }

    fn bounds(&self) -> &treemap::Rect {
        &self.rect
    }

    fn set_bounds(&mut self, bounds: treemap::Rect) {
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
            rect: treemap::Rect::new(),
        })
    }
}
