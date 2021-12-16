pub const TBL_EMAILS: &str = r#"
CREATE TABLE IF NOT EXISTS emails (
  path TEXT NOT NULL,
  sender_domain TEXT NOT NULL,
  sender_local_part TEXT NOT NULL,
  sender_name TEXT NOT NULL,
  year INTEGER NOT NULL,
  month INTEGER NOT NULL,
  day INTEGER NOT NULL,
  timestamp INTEGER NOT NULL,
  subject TEXT NOT NULL,
  to_count INTEGER NOT NULL,
  to_group TEXT NULL,
  to_name TEXT NULL,
  to_address TEXT NULL,
  is_reply BOOL,
  is_send BOOL,
  meta_tags TEXT NULL,
  meta_is_seen BOOL NULL
);"#;

pub const QUERY_EMAILS: &str = r#"
INSERT INTO emails
    (
        path, sender_domain, sender_local_part, sender_name,
        year, month, day, timestamp, subject,
        to_count, to_group, to_name, to_address,
        is_reply, is_send,
        meta_tags, meta_is_seen
    )
VALUES
    (
        ?, ?, ?, ?,
        ?, ?, ?, ?, ?,
        ?, ?, ?, ?,
        ?, ?,
        ?, ?
    )
"#;

pub const TBL_ERRORS: &str = r#"
CREATE TABLE IF NOT EXISTS errors (
  message TEXT NOT NULL
);"#;

pub const QUERY_ERRORS: &str = r#"
INSERT INTO errors
    (message)
VALUES
    (?)
"#;

pub const TBL_META: &str = r#"
CREATE TABLE IF NOT EXISTS meta (
    key TEXT NOT NULL,
    value TEXT NOT NULL
);"#;

pub const QUERY_INSERT_META: &str = r#"
INSERT INTO meta
    (key, value)
VALUES
    (?, ?)
"#;

pub const QUERY_SELECT_META: &str = r#"
SELECT key, value FROM meta"#;

pub const QUERY_COUNT_MAILS: &str = r#"
SELECT count(path) FROM emails
"#;
