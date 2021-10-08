use std::collections::HashMap;
use std::convert::TryInto;
use std::str::FromStr;

use chrono::prelude::*;
use eyre::{bail, eyre, Result};
use rusqlite::{self, types, Row};
use serde_json::Value;

use super::query::{Field, ValueField, AMOUNT_FIELD_NAME};
use super::query_result::QueryResult;
use crate::importer::{EmailEntry, EmailMeta};

/// rusqlite does offer Serde to Value conversion, but it
/// converts everything to strings!
pub fn json_to_value(input: &Value) -> Result<types::Value> {
    let ok = match input {
        Value::Number(n) if n.is_i64() => {
            types::Value::Integer(n.as_i64().ok_or(eyre!("Invalid Number {:?}", n))?)
        }
        Value::Number(n) if n.is_u64() => {
            let value = n.as_u64().ok_or(eyre!("Invalid Number {:?}", n))?;
            let converted: i64 = value.try_into()?;
            types::Value::Integer(converted)
        }
        Value::Number(n) if n.is_f64() => {
            types::Value::Real(n.as_f64().ok_or(eyre!("Invalid Number {:?}", n))?)
        }
        Value::Bool(n) => types::Value::Integer(*n as i64),
        Value::String(n) => types::Value::Text(n.clone()),
        _ => bail!("Invalid type: {}", &input),
    };
    Ok(ok)
}

pub trait RowConversion<'a>: Sized {
    fn grouped_from_row<'stmt>(field: &'a Field, row: &Row<'stmt>) -> Result<Self>;
    fn from_row<'stmt>(fields: &'a [Field], row: &Row<'stmt>) -> Result<Self>;
}

impl<'a> RowConversion<'a> for QueryResult {
    fn grouped_from_row<'stmt>(field: &'a Field, row: &Row<'stmt>) -> Result<Self> {
        let amount: usize = row.get(AMOUNT_FIELD_NAME)?;
        let values = values_from_fields(&[*field], &row)?;

        Ok(QueryResult::Grouped {
            count: amount,
            value: values[field].clone(),
        })
    }
    fn from_row<'stmt>(fields: &'a [Field], row: &Row<'stmt>) -> Result<Self> {
        let values = values_from_fields(&fields, &row)?;
        Ok(QueryResult::Normal(values))
    }
}

fn values_from_fields<'stmt>(
    fields: &[Field],
    row: &Row<'stmt>,
) -> Result<HashMap<Field, ValueField>> {
    let mut values: HashMap<Field, ValueField> = HashMap::default();
    for field in fields {
        use Field::*;
        // Use type safety when unpacking
        match field {
            SenderDomain | SenderLocalPart | SenderName | ToGroup | ToName | ToAddress
            | Subject => {
                let string: String = row.get::<&str, String>(field.as_str())?.into();
                values.insert(*field, ValueField::string(&field, &string));
            }
            Year | Month | Day => {
                values.insert(
                    *field,
                    ValueField::usize(&field, row.get::<&str, usize>(field.as_str())?.into()),
                );
            }
            IsReply | IsSend => {
                values.insert(
                    *field,
                    ValueField::bool(&field, row.get::<&str, bool>(field.as_str())?.into()),
                );
            }
        }
    }
    Ok(values)
}

impl EmailEntry {
    #[allow(unused)]
    fn from_row<'stmt>(row: &Row<'stmt>) -> Result<Self> {
        let path: String = row.get("path")?;
        let path = std::path::PathBuf::from_str(&path)?;
        let sender_domain: String = row.get("sender_domain")?;
        let sender_local_part: String = row.get("sender_local_part")?;
        let sender_name: String = row.get("sender_name")?;
        let timestamp: i64 = row.get("timestamp")?;
        let datetime = Utc.timestamp(timestamp, 0);
        let subject: String = row.get("subject")?;
        let to_count: usize = row.get("to_count")?;
        let to_group: Option<String> = row.get("to_group")?;
        let to_name: Option<String> = row.get("to_name")?;
        let to_address: Option<String> = row.get("to_address")?;

        let to_first = to_address.map(|a| (to_name.unwrap_or_default(), a));

        let is_reply: bool = row.get("is_reply")?;
        let is_send: bool = row.get("is_send")?;

        // Parse EmailMeta
        let meta_tags: Option<String> = row.get("meta_tags")?;
        let meta_is_seen: Option<bool> = row.get("meta_is_seen")?;
        let meta = match (meta_tags, meta_is_seen) {
            (Some(a), Some(b)) => Some(EmailMeta::from(b, &a)),
            _ => None,
        };

        Ok(EmailEntry {
            path,
            sender_domain,
            sender_local_part,
            sender_name,
            datetime,
            subject,
            to_count,
            to_group,
            to_first,
            is_reply,
            is_send,
            meta,
        })
    }
}
