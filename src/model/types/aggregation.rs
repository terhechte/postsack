use crate::database::query::{Field, ValueField};

/// A aggregation field.
/// Contains the `Field` to aggregate by, the `Value` used for aggregation
/// As well as the index in the stack of Segmentations that this relates to.
pub struct Aggregation {
    pub(in super::super) value: Option<ValueField>,
    pub(in super::super) field: Field,
    pub(in super::super) index: usize,
}

impl Aggregation {
    /// Return the value in this aggregation as a string
    pub fn value(&self) -> Option<String> {
        self.value.as_ref().map(|e| e.value().to_string())
    }

    /// The name of the field as a `String`
    pub fn name(&self) -> &str {
        self.field.name()
    }

    /// The indes of the field within the given fields
    pub fn index(&self, in_fields: &[Field]) -> Option<usize> {
        in_fields.iter().position(|p| p == &self.field)
    }
}
