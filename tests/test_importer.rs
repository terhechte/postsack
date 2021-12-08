use postsack::{
    self,
    database::{query, Database},
    importer::Importerlike,
    types::FormatType,
};

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use postsack::database::{query::Field, query_result::QueryResult};

    use super::*;

    #[test]
    /// Test that the mbox importer works
    fn test_mbox_import() {
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

        let subjects: HashSet<String> = HashSet::from(
            mails
                .into_iter()
                .map(|s| match s {
                    QueryResult::Normal(row) => row[&Field::Subject].to_string(),
                    _ => panic!(),
                })
                .collect(),
        );
        assert!(subjects.contains(" check bogus body header (from)"));
    }

    /// Test that the AppleMail importer works
    #[test]
    fn test_applemail_importer() {
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
