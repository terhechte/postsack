//! # Database
//!
//! A implementation of the [`ps_core::DatabaseLike`] trait on top of SQLite.

mod conversion;
mod db;
mod sql;

pub use conversion::{value_from_field, RowConversion};
pub use db::Database;
