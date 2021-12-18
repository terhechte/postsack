use std::collections::HashMap;
use std::convert::TryInto;

use ps_core::eyre::{bail, eyre, Result};
use ps_core::Value;
use rusqlite::{self, types, Row};

use ps_core::{EmailMeta, Field, QueryResult, ValueField, AMOUNT_FIELD_NAME};

/// rusqlite does offer Serde to Value conversion, but it
/// converts everything to strings!
pub fn json_to_value(input: &Value) -> Result<types::Value> {
    let ok = match input {
        Value::Number(n) if n.is_i64() => {
            types::Value::Integer(n.as_i64().ok_or_else(|| eyre!("Invalid Number {:?}", n))?)
        }
        Value::Number(n) if n.is_u64() => {
            let value = n.as_u64().ok_or_else(|| eyre!("Invalid Number {:?}", n))?;
            let converted: i64 = value.try_into()?;
            types::Value::Integer(converted)
        }
        Value::Number(n) if n.is_f64() => {
            types::Value::Real(n.as_f64().ok_or_else(|| eyre!("Invalid Number {:?}", n))?)
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
        let values = values_from_fields(&[*field], row)?;

        Ok(QueryResult::Grouped {
            count: amount,
            value: values[field].clone(),
        })
    }
    fn from_row<'stmt>(fields: &'a [Field], row: &Row<'stmt>) -> Result<Self> {
        let values = values_from_fields(fields, row)?;
        Ok(QueryResult::Normal(values))
    }
}

fn values_from_fields<'stmt>(
    fields: &[Field],
    row: &Row<'stmt>,
) -> Result<HashMap<Field, ValueField>> {
    let mut values: HashMap<Field, ValueField> = HashMap::default();
    for field in fields {
        values.insert(*field, value_from_field(field, row)?);
    }
    Ok(values)
}

pub fn value_from_field<'stmt>(field: &Field, row: &Row<'stmt>) -> Result<ValueField> {
    use Field::*;
    // Use type safety when unpacking
    match field {
        Path | SenderDomain | SenderLocalPart | SenderName | ToGroup | ToName | ToAddress
        | Subject => {
            let string: String = row.get::<&str, String>(field.as_str())?;
            Ok(ValueField::string(field, &string))
        }
        Year | Month | Day | Timestamp => {
            return Ok(ValueField::usize(
                field,
                row.get::<&str, usize>(field.as_str())?,
            ));
        }
        MetaTags => {
            let tag_string = row.get::<&str, String>(field.as_str())?;
            let tags = EmailMeta::tags_from_string(&tag_string);
            Ok(ValueField::array(
                field,
                tags.into_iter().map(Value::String).collect(),
            ))
        }
        IsReply | IsSend | MetaIsSeen => {
            return Ok(ValueField::bool(
                field,
                row.get::<&str, bool>(field.as_str())?,
            ));
        }
    }
}
