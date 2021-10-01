use std::str::FromStr;

use chrono::prelude::*;
use eyre::Result;
use rusqlite::{self, Row};

use super::query::{GroupByField, ValueField};
use super::query_result::QueryResult;
use crate::types::{EmailEntry, EmailMeta};

pub trait RowConversion<'a>: Sized {
    fn grouped_from_row<'stmt>(fields: &'a [GroupByField], row: &Row<'stmt>) -> Result<Self>;
}

impl<'a> RowConversion<'a> for QueryResult<'a> {
    fn grouped_from_row<'stmt>(fields: &'a [GroupByField], row: &Row<'stmt>) -> Result<Self> {
        let amount: usize = row.get("amount")?;

        let mut values = vec![];
        for field in fields {
            use GroupByField::*;
            match field {
                // Str fields
                SenderDomain => values.push(ValueField::SenderDomain(
                    row.get::<&str, String>(field.into())?.into(),
                )),
                SenderLocalPart => values.push(ValueField::SenderLocalPart(
                    row.get::<&str, String>(field.into())?.into(),
                )),
                SenderName => values.push(ValueField::SenderName(
                    row.get::<&str, String>(field.into())?.into(),
                )),
                ToGroup => values.push(ValueField::ToGroup(
                    row.get::<&str, String>(field.into())?.into(),
                )),
                ToName => values.push(ValueField::ToName(
                    row.get::<&str, String>(field.into())?.into(),
                )),
                ToAddress => values.push(ValueField::ToAddress(
                    row.get::<&str, String>(field.into())?.into(),
                )),

                // usize field
                Year => values.push(ValueField::Year(
                    row.get::<&str, usize>(field.into())?.into(),
                )),
                Month => values.push(ValueField::Day(
                    row.get::<&str, usize>(field.into())?.into(),
                )),
                Day => values.push(ValueField::Day(
                    row.get::<&str, usize>(field.into())?.into(),
                )),

                // bool field
                IsReply => values.push(ValueField::IsReply(
                    row.get::<&str, bool>(field.into())?.into(),
                )),
                IsSend => values.push(ValueField::IsSend(
                    row.get::<&str, bool>(field.into())?.into(),
                )),
            }
        }

        Ok(QueryResult {
            count: amount,
            values,
        })
    }
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
