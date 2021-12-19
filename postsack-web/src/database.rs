use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::thread::JoinHandle;
use std::{ops::Range, path::Path};

use ps_core::{
    crossbeam_channel::Sender,
    eyre::{bail, Result},
    Config, DBMessage, DatabaseLike, DatabaseQuery, Field, Filter, Query, QueryResult, Value,
    ValueField,
};
use ps_core::{OtherQuery, QueryRow};

#[derive(Default, Clone)]
pub struct Entry {
    pub sender_domain: &'static str,
    pub sender_local_part: &'static str,
    pub sender_name: &'static str,
    pub year: usize,
    pub month: usize,
    pub day: usize,
    pub timestamp: usize,
    pub subject: &'static str,
    pub to_name: &'static str,
    pub to_address: &'static str,
    pub is_reply: bool,
    pub is_send: bool,
}

impl Entry {
    fn value(&self, field: &Field) -> Value {
        match field {
            Field::Path => Value::String("".to_string()),
            Field::SenderDomain => Value::String(self.sender_domain.to_string()),
            Field::SenderLocalPart => Value::String(self.sender_local_part.to_string()),
            Field::SenderName => Value::String(self.sender_name.to_string()),
            Field::Subject => Value::String(self.subject.to_string()),
            Field::ToName => Value::String(self.to_name.to_string()),
            Field::ToAddress => Value::String(self.to_address.to_string()),
            Field::ToGroup => Value::String("".to_string()),

            Field::Year => Value::Number(self.year.into()),
            Field::Month => Value::Number(self.month.into()),
            Field::Day => Value::Number(self.day.into()),
            Field::Timestamp => Value::Number(self.timestamp.into()),

            Field::IsReply => Value::Bool(self.is_reply),
            Field::IsSend => Value::Bool(self.is_send),

            Field::MetaIsSeen => Value::Bool(false),
            Field::MetaTags => Value::Array(Vec::new()),
        }
    }

    fn as_row(&self, fields: &[Field]) -> QueryRow {
        let mut row = QueryRow::new();
        for field in fields {
            let value = self.value(field);
            let value_field = ValueField::new(field, value);
            row.insert(*field, value_field);
        }
        row
    }
}

#[derive(Debug, PartialEq, Eq)]
struct HashedValue(Value);

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for HashedValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.0 {
            Value::String(s) => s.hash(state),
            Value::Number(s) => s.hash(state),
            Value::Array(s) => {
                format!("{:?}", s).hash(state);
            }
            Value::Bool(s) => s.hash(state),
            _ => {
                format!("{:?}", &self.0).hash(state);
            }
        }
    }
}

pub struct FakeDatabase;

impl FakeDatabase {
    #[allow(unused)]
    pub fn total_item_count() -> usize {
        ENTRIES.len()
    }

    fn query_normal(
        &self,
        fields: &[Field],
        filters: &[Filter],
        range: &Range<usize>,
    ) -> Vec<QueryResult> {
        let entries = self.filtered(filters);
        let mut result = Vec::new();
        for entry in entries.skip(range.start).take(range.end) {
            result.push(QueryResult::Normal(entry.as_row(fields)));
        }
        result
    }

    fn query_grouped(&self, filters: &[Filter], group_by: &Field) -> Vec<QueryResult> {
        let mut map = HashMap::<HashedValue, usize>::new();
        for entry in self
            .filtered(filters)
            .map(|e| HashedValue(e.value(group_by)))
        {
            let entry = map.entry(entry).or_insert(0);
            *entry += 1;
        }

        let mut result = Vec::new();
        for (key, value) in map {
            result.push(QueryResult::Grouped {
                value: ValueField::new(group_by, key.0),
                count: value,
            })
        }
        result
    }

    fn query_other(&self, field: &Field) -> Vec<QueryResult> {
        let mut set = HashSet::<HashedValue>::new();
        for entry in &ENTRIES {
            let hashed_entry = HashedValue(entry.value(field));
            if !set.contains(&hashed_entry) {
                set.insert(hashed_entry);
            }
        }

        let mut result = Vec::new();
        for value in set {
            result.push(QueryResult::Other(ValueField::new(field, value.0)));
        }
        result
    }

    fn filtered<'a>(&'a self, filters: &'a [Filter]) -> impl Iterator<Item = &'a Entry> {
        ENTRIES.iter().filter(move |entry| {
            for filter in filters {
                // Go through all filters and escape early if they don't match
                match filter {
                    Filter::Like(vf) => {
                        let other = entry.value(vf.field());
                        if vf.value() != &other {
                            return false;
                        }
                    }
                    Filter::NotLike(vf) => {
                        let other = entry.value(vf.field());
                        if vf.value() == &other {
                            return false;
                        }
                    }
                    Filter::Contains(vf) => {
                        let other = entry.value(vf.field());
                        match (&other, vf.value()) {
                            (Value::String(a), Value::String(b)) => {
                                if !a.contains(b) {
                                    return false;
                                }
                            }
                            _ => {
                                let s1 = format!("{}", vf.value());
                                let s2 = format!("{}", &other);
                                if !s2.contains(&s1) {
                                    return false;
                                }
                            }
                        }
                    }
                    Filter::Is(vf) => {
                        let other = entry.value(vf.field());
                        if vf.value() != &other {
                            return false;
                        }
                    }
                }
            }
            true
        })
    }
}

impl Clone for FakeDatabase {
    fn clone(&self) -> Self {
        FakeDatabase
    }
}

impl DatabaseQuery for FakeDatabase {
    fn query(&self, query: &Query) -> Result<Vec<QueryResult>> {
        match query {
            Query::Normal {
                fields,
                filters,
                range,
            } => Ok(self.query_normal(fields, filters, range)),
            Query::Grouped { filters, group_by } => Ok(self.query_grouped(filters, group_by)),
            Query::Other {
                query: OtherQuery::All(q),
            } => Ok(self.query_other(q)),
        }
    }
}

impl DatabaseLike for FakeDatabase {
    fn new(_path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(FakeDatabase {})
    }

    fn config(_path: impl AsRef<Path>) -> Result<Config>
    where
        Self: Sized,
    {
        bail!("Na")
    }

    fn total_mails(&self) -> Result<usize> {
        Ok(ENTRIES.len())
    }

    fn import(self) -> (Sender<DBMessage>, JoinHandle<Result<usize>>) {
        panic!()
    }
    fn save_config(&self, _config: Config) -> Result<()> {
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
use super::generated::ENTRIES;

#[cfg(not(target_arch = "wasm32"))]
const ENTRIES: [Entry; 7] = [
    Entry {
        sender_local_part: "tellus.non.magna",
        is_send: true,
        to_address: "john@doe.com",
        sender_name: "Sybill Fleming",
        timestamp: 1625731134,
        month: 12,
        to_name: "",
        is_reply: false,
        year: 2013,
        sender_domain: "protonmail.edu",
        day: 28,
        subject: "libero et tristique pellentesque, tellus sem mollis dui,",
    },
    Entry {
        sender_local_part: "mauris.sapien",
        is_send: true,
        to_address: "john@doe.com",
        sender_name: "Ignatius Reed",
        timestamp: 1645571678,
        month: 10,
        to_name: "",
        is_reply: true,
        year: 2020,
        sender_domain: "icloud.com",
        day: 26,
        subject: "nisi magna sed dui. Fusce aliquam,",
    },
    Entry {
        sender_local_part: "magna.nam",
        is_send: false,
        to_address: "john@doe.com",
        sender_name: "Geraldine Gay",
        timestamp: 1631684202,
        month: 8,
        to_name: "",
        is_reply: true,
        year: 2016,
        sender_domain: "aol.org",
        day: 18,
        subject: "semper auctor. Mauris vel turpis. Aliquam adipiscing",
    },
    Entry {
        sender_local_part: "tortor",
        is_send: true,
        to_address: "john@doe.com",
        sender_name: "Colt Clark",
        timestamp: 1640866204,
        month: 4,
        to_name: "",
        is_reply: true,
        year: 2012,
        sender_domain: "aol.ca",
        day: 2,
        subject: "hendrerit id, ante. Nunc mauris sapien, cursus",
    },
    Entry {
        sender_local_part: "urna.convallis.erat",
        is_send: true,
        to_address: "john@doe.com",
        sender_name: "Joy Clark",
        timestamp: 1646836804,
        month: 2,
        to_name: "",
        is_reply: true,
        year: 2020,
        sender_domain: "protonmail.ca",
        day: 10,
        subject: "dui nec urna suscipit nonummy. Fusce fermentum fermentum arcu. Vestibulum",
    },
    Entry {
        sender_local_part: "amet.luctus",
        is_send: false,
        to_address: "john@doe.com",
        sender_name: "Ray Bowers",
        timestamp: 1609958850,
        month: 6,
        to_name: "",
        is_reply: false,
        year: 2015,
        sender_domain: "protonmail.org",
        day: 30,
        subject: "turpis egestas. Aliquam fringilla cursus",
    },
    Entry {
        sender_local_part: "vehicula.et",
        is_send: true,
        to_address: "john@doe.com",
        sender_name: "Maris Shaw",
        timestamp: 1612463990,
        month: 10,
        to_name: "",
        is_reply: false,
        year: 2018,
        sender_domain: "hotmail.ca",
        day: 30,
        subject: "molestie orci tincidunt",
    },
];
