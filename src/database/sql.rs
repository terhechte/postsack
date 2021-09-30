pub const TBL_EMAILS: &str = r#"
CREATE TABLE IF NOT EXISTS emails (
  path TEXT NOT NULL,
  domain TEXT NOT NULL,
  local_part TEXT NOT NULL,
  year INTEGER NOT NULL,
  month INTEGER NOT NULL,
  day INTEGER NOT NULL,
  subject TEXT NOT NULL
);"#;

pub const TBL_ERRORS: &str = r#"
CREATE TABLE IF NOT EXISTS errors (
  message TEXT NOT NULL,
  path TEXT NOT NULL
);"#;

pub const QUERY_EMAILS: &str = r#"
INSERT INTO emails
    (path, domain, local_part, year, month, day, subject)
VALUES
    (?, ?, ?, ?, ?, ?, ?)
"#;

pub const QUERY_ERRORS: &str = r#"
INSERT INTO errors
    (message, path)
VALUES
    (?, ?)
"#;
