use rsql_builder;
use serde_json::{self, Value};
use strum::{self, IntoEnumIterator};
use strum_macros::{EnumIter, IntoStaticStr};

pub const AMOUNT_FIELD_NAME: &str = "amount";

pub enum Filter {
    Like(ValueField),
    NotLike(ValueField),
    Is(ValueField),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum Field {
    SenderDomain,
    SenderLocalPart,
    SenderName,
    Year,
    Month,
    Day,
    ToGroup,
    ToName,
    ToAddress,
    IsReply,
    IsSend,
}

impl Field {
    pub fn all_cases() -> impl Iterator<Item = Field> {
        Field::iter()
    }

    /// Just a wrapper to offer `into` without the type ambiguity
    /// that sometimes arises
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ValueField {
    field: Field,
    value: Value,
}

impl ValueField {
    pub fn string<S: AsRef<str>>(field: &Field, value: S) -> ValueField {
        ValueField {
            field: field.clone(),
            value: Value::String(value.as_ref().to_string()),
        }
    }

    pub fn bool(field: &Field, value: bool) -> ValueField {
        ValueField {
            field: field.clone(),
            value: Value::Bool(value),
        }
    }

    pub fn usize(field: &Field, value: usize) -> ValueField {
        ValueField {
            field: field.clone(),
            value: Value::Number(value.into()),
        }
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

pub enum Query<'a> {
    Grouped {
        filters: &'a [Filter],
        group_by: &'a Field,
    },
    Normal {
        fields: &'a [Field],
        filters: &'a [Filter],
    },
}

impl<'a> Query<'a> {
    fn filters(&self) -> &'a [Filter] {
        match self {
            &Query::Grouped { filters, .. } => filters,
            &Query::Normal { filters, .. } => filters,
        }
    }
}

impl<'a> Query<'a> {
    pub fn to_sql(&self) -> (String, Vec<serde_json::Value>) {
        let mut conditions = {
            let mut whr = rsql_builder::B::new_where();
            for filter in self.filters() {
                match filter {
                    Filter::Like(f) => whr.like(f.field.into(), f.value()),
                    Filter::NotLike(f) => whr.not_like(f.field.into(), f.value()),
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
            Query::Normal { fields, .. } => {
                let fields: Vec<&str> = fields.iter().map(|e| e.into()).collect();
                (
                    format!("SELECT {} FROM emails", fields.join(", ")),
                    "".to_owned(),
                )
            }
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
            filters: &[
                Filter::Like(ValueField::string(&Field::SenderDomain, "gmail.com")),
                Filter::Is(ValueField::usize(&Field::Year, 2021)),
            ],
            group_by: &Field::Month,
        };
        dbg!(&query.to_sql());
    }
}
