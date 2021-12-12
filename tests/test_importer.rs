use postsack::{
    self,
    database::{query, Database},
    importer::Importerlike,
    types::FormatType,
};


#[cfg(test)]
mod tests {
    use postsack::database::{query::Field, query_result::QueryResult};
    use std::sync::Once;

    use super::*;

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", "trace");
            }
            postsack::setup_tracing();
        });
    }

    #[test]
    /// Test that the mbox importer works
    fn test_mbox_import() {
        initialize();
        let path = "tests/resources/mbox";
        let config =
            postsack::types::Config::new(None, path, vec!["".to_string()], FormatType::Mbox)
                .expect("Config");
        let importer = postsack::importer::mbox_importer(config.clone());
        let (_receiver, handle) = importer.import().unwrap();
        handle.join().expect("").expect("");

        // The temporary database path
        let db = Database::new(&config.database_path).unwrap();

        let total_mails = db.total_mails().expect("Expected total mails");
        assert_eq!(total_mails, 141);

        let mails = db.query(&query::Query::Normal {
            fields: vec![query::Field::Subject],
            filters: Vec::new(),
            range: 0..141,
        });
        let mails = mails.expect("Expected Mails");

        #[allow(clippy::needless_collect)]
        let subjects: Vec<String> = mails
            .into_iter()
            .map(|s| match s {
                QueryResult::Normal(row) => row[&Field::Subject].to_string(),
                _ => panic!(),
            })
            .collect();
        assert!(subjects.contains(&" check bogus body header (from)".into()));
    }

    /// Test that the AppleMail importer works
    #[test]
    /// FIXME: On windows we have an issue with the `\n` / `\r\n` line endings it seems
    /// Disabling it for now
    #[cfg(not(target_os = "windows"))]
    fn test_applemail_importer() {
        initialize();
        let path = "tests/resources/applemail";
        let config =
            postsack::types::Config::new(None, path, vec!["".to_string()], FormatType::AppleMail)
                .expect("Config");
        let importer = postsack::importer::applemail_importer(config.clone());
        let (_receiver, handle) = importer.import().unwrap();
        handle.join().expect("").expect("");
        // The temporary database path
        let db = Database::new(&config.database_path).unwrap();
        let mails = db.query(&query::Query::Normal {
            fields: vec![query::Field::Subject],
            filters: Vec::new(),
            range: 0..10,
        });
        let mails = mails.expect("Expected Mails");
        assert_eq!(mails.len(), 4);
    }
}
