use rsql_builder;
use serde_json;

/// For In-Queries, we need a Vec of at least one, so we make a new type
pub struct VecOfMinOne<T> {
    inner: Vec<T>,
}

impl<T> VecOfMinOne<T> {
    /// Create a new `VecOfMinOne`.
    /// If `from` is empty, return `None`
    pub fn new(from: Vec<T>) -> Option<Self> {
        if from.is_empty() {
            return None;
        }
        Some(Self { inner: from })
    }
}

pub enum Filter {
    Like(ValueField),
    NotLike(ValueField),
    Is(ValueField),
    In(VecOfMinOne<ValueField>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GroupByField {
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

impl<'a> ValueField {
    pub fn as_field(&self) -> GroupByField {
        use GroupByField::*;
        match self {
            ValueField::SenderDomain(_) => SenderDomain,
            ValueField::SenderLocalPart(_) => SenderLocalPart,
            ValueField::SenderName(_) => SenderName,
            ValueField::Year(_) => Year,
            ValueField::Month(_) => Month,
            ValueField::Day(_) => Day,
            ValueField::ToGroup(_) => ToGroup,
            ValueField::ToName(_) => ToName,
            ValueField::ToAddress(_) => ToAddress,
            ValueField::IsReply(_) => IsReply,
            ValueField::IsSend(_) => IsSend,
        }
    }
}

/*impl GroupByField {
    pub fn make_str<'a>(value: &'a str, field: GroupByField) -> ValueField<'a> {
        use GroupByField::*;
        match field {
            SenderDomain => ValueField::SenderDomain(value.into()),
            _ => panic!(),
        }
    }
}*/

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ValueField {
    SenderDomain(String),
    SenderLocalPart(String),
    SenderName(String),
    Year(usize),
    Month(usize),
    Day(usize),
    ToGroup(String),
    ToName(String),
    ToAddress(String),
    IsReply(bool),
    IsSend(bool),
}

// FIXME: Maybe use `json-value` instead?
impl ValueField {
    pub fn value(&self) -> Value {
        match (self.is_bool(), self.is_str(), self.is_usize()) {
            (true, false, false) => Value::Bool(*self.as_bool()),
            (false, true, false) => Value::String(self.as_str().to_string()),
            (false, false, true) => Value::Number(*self.as_usize()),
            _ => panic!("Invalid field: {:?}", &self),
        }
    }
}

#[derive(Debug, Hash, Clone)]
pub enum Value {
    Number(usize),
    String(String),
    Bool(bool),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => f.write_str(&n.to_string()),
            Value::Bool(n) => f.write_str(&n.to_string()),
            Value::String(n) => f.write_str(&n),
        }
    }
}

pub trait DynamicType<'a> {
    type BoolType;
    type StrType;
    type UsizeType;
    fn is_str(&self) -> bool;
    fn is_bool(&self) -> bool;
    fn is_usize(&self) -> bool;
    fn as_str(&'a self) -> Self::StrType;
    fn as_bool(&'a self) -> Self::BoolType;
    fn as_usize(&'a self) -> Self::UsizeType;
}

impl<'a> DynamicType<'a> for ValueField {
    type BoolType = &'a bool;
    type StrType = &'a str;
    type UsizeType = &'a usize;

    fn is_str(&self) -> bool {
        !self.is_bool() && !self.is_usize()
    }
    fn is_bool(&self) -> bool {
        use ValueField::*;
        match self {
            IsReply(_) | IsSend(_) => true,
            _ => false,
        }
    }
    fn is_usize(&self) -> bool {
        use ValueField::*;
        match self {
            Year(_) | Month(_) | Day(_) => true,
            _ => false,
        }
    }

    fn as_str(&'a self) -> Self::StrType {
        use ValueField::*;
        match self {
            SenderDomain(a) | SenderLocalPart(a) | SenderName(a) | ToGroup(a) | ToName(a)
            | ToAddress(a) => &a,
            _ => panic!(),
        }
    }

    fn as_bool(&'a self) -> Self::BoolType {
        use ValueField::*;
        match self {
            IsReply(a) | IsSend(a) => a,
            _ => panic!(),
        }
    }

    fn as_usize(&'a self) -> Self::UsizeType {
        use ValueField::*;
        match self {
            Year(a) | Month(a) | Day(a) => a,
            _ => panic!(),
        }
    }
}

impl<'a> DynamicType<'a> for &VecOfMinOne<ValueField> {
    type BoolType = Vec<bool>;
    type StrType = Vec<&'a str>;
    type UsizeType = Vec<usize>;
    fn is_str(&self) -> bool {
        self.inner[0].is_str()
    }
    fn is_bool(&self) -> bool {
        self.inner[0].is_bool()
    }
    fn is_usize(&self) -> bool {
        self.inner[0].is_usize()
    }
    fn as_str(&'a self) -> Self::StrType {
        self.inner.iter().map(|e| e.as_str()).collect()
    }
    fn as_bool(&'a self) -> Self::BoolType {
        self.inner.iter().map(|e| *e.as_bool()).collect()
    }
    fn as_usize(&'a self) -> Self::UsizeType {
        self.inner.iter().map(|e| *e.as_usize()).collect()
    }
}

impl<'a> From<&VecOfMinOne<ValueField>> for &'a str {
    fn from(vector: &VecOfMinOne<ValueField>) -> Self {
        use ValueField::*;
        match &vector.inner[0] {
            SenderDomain(_) => "sender_domain",
            SenderLocalPart(_) => "sender_local_part",
            SenderName(_) => "sender_name",
            Year(_) => "year",
            Month(_) => "month",
            Day(_) => "day",
            ToGroup(_) => "to_group",
            ToName(_) => "to_name",
            ToAddress(_) => "to_address",
            IsReply(_) => "is_reply",
            IsSend(_) => "is_send",
        }
    }
}

impl<'a> From<&'a ValueField> for &'a str {
    fn from(field: &'a ValueField) -> Self {
        use ValueField::*;
        match field {
            SenderDomain(_) => "sender_domain",
            SenderLocalPart(_) => "sender_local_part",
            SenderName(_) => "sender_name",
            Year(_) => "year",
            Month(_) => "month",
            Day(_) => "day",
            ToGroup(_) => "to_group",
            ToName(_) => "to_name",
            ToAddress(_) => "to_address",
            IsReply(_) => "is_reply",
            IsSend(_) => "is_send",
        }
    }
}

impl From<&GroupByField> for &str {
    fn from(field: &GroupByField) -> Self {
        use GroupByField::*;
        match field {
            SenderDomain => "sender_domain",
            SenderLocalPart => "sender_local_part",
            SenderName => "sender_name",
            Year => "year",
            Month => "month",
            Day => "day",
            ToGroup => "to_group",
            ToName => "to_name",
            ToAddress => "to_address",
            IsReply => "is_reply",
            IsSend => "is_send",
        }
    }
}

pub struct Query<'a> {
    pub filters: &'a [Filter],
    pub group_by: &'a [GroupByField],
}

impl<'a> Query<'a> {
    pub fn to_sql(&self) -> (String, Vec<serde_json::Value>) {
        let mut conditions = {
            let mut whr = rsql_builder::B::new_where();
            for filter in self.filters {
                match filter {
                    // Bool
                    Filter::Like(f) if f.is_bool() => whr.like(f.into(), f.as_bool()),
                    Filter::NotLike(f) if f.is_bool() => whr.not_like(f.into(), f.as_bool()),
                    Filter::In(f) if f.is_bool() => whr.r#in(f.into(), &f.as_bool()),
                    Filter::Is(f) if f.is_bool() => whr.eq(f.into(), f.as_bool()),

                    // usize
                    Filter::Like(f) if f.is_usize() => whr.like(f.into(), f.as_usize()),
                    Filter::NotLike(f) if f.is_usize() => whr.not_like(f.into(), f.as_usize()),
                    Filter::In(f) if f.is_usize() => whr.r#in(f.into(), &f.as_usize()),
                    Filter::Is(f) if f.is_usize() => whr.eq(f.into(), f.as_usize()),

                    // str
                    Filter::Like(f) if f.is_str() => whr.like(f.into(), &f.as_str()),
                    Filter::NotLike(f) if f.is_str() => whr.not_like(f.into(), &f.as_str()),
                    Filter::In(f) if f.is_str() => whr.r#in(f.into(), &f.as_str()),
                    Filter::Is(f) if f.is_str() => whr.eq(f.into(), &f.as_str()),

                    _ => &whr,
                };
            }
            whr
        };

        let group_by_fields: Vec<&str> = self.group_by.iter().map(|e| e.into()).collect();
        let group_by = format!("GROUP BY {}", &group_by_fields.join(", "));

        // If we have a group by, we always include the count
        let header = if self.group_by.is_empty() {
            "SELECT * FROM emails".to_owned()
        } else {
            format!(
                "SELECT count(path) as amount, {} FROM emails",
                group_by_fields.join(", ")
            )
        };

        let (sql, values) = rsql_builder::B::prepare(
            rsql_builder::B::new_sql(&header)
                .push_build(&mut conditions)
                .push_sql(&group_by),
        );

        dbg!(&sql);
        dbg!(&values);

        (sql, values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test() {
        let value = format!("bx");
        let query = Query {
            filters: &[
                Filter::Is(ValueField::ToName("bam".into())),
                Filter::Like(ValueField::SenderName(value.into())),
                Filter::Like(ValueField::Year(2323)),
            ],
            group_by: &[GroupByField::Month],
        };
        dbg!(&query.to_sql());
    }
}
