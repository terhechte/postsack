use rusqlite::{self, Error, Row};

pub trait RowConversion: Sized {
    fn from_row<'stmt>(row: &Row<'stmt>) -> Result<Self, Error>;
}

/*impl RowConversion for EmailEntry {
fn from_row<'stmt>(row: &Row<'stmt>) -> Result<Self, Error> {
    let path: String = row.get("path")?;
    let domain: String = row.get("domain")?;
    let local_part: String = row.get("local_part")?;
    let year: usize = row.get("year")?;
    let month: usize = row.get("month")?;
    let day: usize = row.get("day")?;
    let created = email_parser::time::DateTime::
    Ok(EmailEntry {
        path, domain, local_part, year, month, day
    })
}
*/
