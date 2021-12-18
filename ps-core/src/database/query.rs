use rsql_builder;
use serde_json;
pub use serde_json::Value;
use strum::{self, IntoEnumIterator};
use strum_macros::{EnumIter, IntoStaticStr};

use std::ops::Range;

pub const AMOUNT_FIELD_NAME: &str = "amount";

#[derive(Clone, Debug)]
pub enum Filter {
    /// A database Like Operation
    Like(ValueField),
    NotLike(ValueField),
    /// A extended like that implies:
    /// - wildcards on both sides (like '%test%')
    /// - case in-sensitive comparison
    /// - Trying to handle values as strings
    Contains(ValueField),
    Is(ValueField),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum Field {
    Path,
    SenderDomain,
    SenderLocalPart,
    SenderName,
    Year,
    Month,
    Day,
    Timestamp,
    ToGroup,
    ToName,
    ToAddress,
    IsReply,
    IsSend,
    Subject,
    MetaIsSeen,
    MetaTags,
}

const INVALID_FIELDS: &[Field] = &[
    Field::Path,
    Field::Subject,
    Field::Timestamp,
    Field::IsReply,
    Field::IsSend,
    Field::MetaIsSeen,
    Field::MetaTags,
];

impl Field {
    pub fn all_cases() -> impl Iterator<Item = Field> {
        Field::iter().filter(|f| !INVALID_FIELDS.contains(f))
    }

    /// Just a wrapper to offer `into` without the type ambiguity
    /// that sometimes arises
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    /// A human readable name
    pub fn name(&self) -> &str {
        use Field::*;
        match self {
            SenderDomain => "Domain",
            SenderLocalPart => "Address",
            SenderName => "Name",
            ToGroup => "Group",
            ToName => "To name",
            ToAddress => "To address",
            Year => "Year",
            Month => "Month",
            Day => "Day",
            Subject => "Subject",
            _ => self.as_str(),
        }
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ValueField {
    field: Field,
    value: Value,
}

impl ValueField {
    pub fn new(field: &Field, value: Value) -> ValueField {
        ValueField {
            field: *field,
            value,
        }
    }

    pub fn string<S: AsRef<str>>(field: &Field, value: S) -> ValueField {
        ValueField {
            field: *field,
            value: Value::String(value.as_ref().to_string()),
        }
    }

    pub fn bool(field: &Field, value: bool) -> ValueField {
        ValueField {
            field: *field,
            value: Value::Bool(value),
        }
    }

    pub fn usize(field: &Field, value: usize) -> ValueField {
        ValueField {
            field: *field,
            value: Value::Number(value.into()),
        }
    }

    pub fn array(field: &Field, value: Vec<Value>) -> ValueField {
        ValueField {
            field: *field,
            value: Value::Array(value),
        }
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match &self.value {
            Value::String(s) => s.clone(),
            _ => format!("{}", &self.value),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OtherQuery {
    /// Get all contents of a specific field
    All(Field),
}

#[derive(Clone, Debug)]
pub enum Query {
    Grouped {
        filters: Vec<Filter>,
        group_by: Field,
    },
    Normal {
        fields: Vec<Field>,
        filters: Vec<Filter>,
        range: Range<usize>,
    },
    Other {
        query: OtherQuery,
    },
}

impl Query {
    fn filters(&self) -> &[Filter] {
        match self {
            Query::Grouped { ref filters, .. } => filters,
            Query::Normal { ref filters, .. } => filters,
            Query::Other { .. } => &[],
        }
    }
}

impl Query {
    pub fn to_sql(&self) -> (String, Vec<serde_json::Value>) {
        let mut conditions = {
            let mut whr = rsql_builder::B::new_where();
            for filter in self.filters() {
                match filter {
                    Filter::Like(f) => whr.like(f.field.into(), f.value()),
                    Filter::NotLike(f) => whr.not_like(f.field.into(), f.value()),
                    Filter::Contains(f) => whr.like(
                        f.field.into(),
                        &format!("%{}%", f.to_string().to_lowercase()),
                    ),
                    Filter::Is(f) => whr.eq(f.field.into(), f.value()),
                };
            }
            whr
        };

        let (header, group_by) = match self {
            Query::Grouped { group_by, .. } => (
                format!(
                    "SELECT count(path) as {}, {} FROM emails",
                    AMOUNT_FIELD_NAME,
                    group_by.as_str()
                ),
                format!("GROUP BY {}", group_by.as_str()),
            ),
            Query::Normal { fields, range, .. } => {
                let fields: Vec<&str> = fields.iter().map(|e| e.into()).collect();
                (
                    format!("SELECT {} FROM emails", fields.join(", ")),
                    format!("LIMIT {}, {}", range.start, range.end - range.start),
                )
            }
            Query::Other {
                query: OtherQuery::All(field),
            } => (
                format!(
                    "SELECT {} FROM emails GROUP BY {}",
                    field.as_str(),
                    field.as_str()
                ),
                format!(""),
            ),
        };

        let (sql, values) = rsql_builder::B::prepare(
            rsql_builder::B::new_sql(&header)
                .push_build(&mut conditions)
                .push_sql(&group_by),
        );

        (sql, values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test() {
        let query = Query::Grouped {
            filters: vec![
                Filter::Like(ValueField::string(&Field::SenderDomain, "gmail.com")),
                Filter::Is(ValueField::usize(&Field::Year, 2021)),
            ],
            group_by: Field::Month,
        };
        dbg!(&query.to_sql());
    }
}
