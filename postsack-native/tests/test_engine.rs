use ps_core::{
    self,
    model::{self, Engine, Rect},
    Config, DatabaseLike, DatabaseQuery, Field, Filter, FormatType, Importerlike, ValueField,
};
use ps_database::Database;
use ps_importer::mbox_importer;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", "trace");
            }
            ps_core::setup_tracing();
        });
    }

    #[test]
    fn test_engine_all() {
        initialize();
        let config = create_database();
        let mut engine = Engine::new::<Database>(&config).expect("Expected Engine");
        engine.start().expect("Expect to start engine");
        engine.wait().expect("Expected working wait");
        let segment = {
            let segments =
                model::segmentations::layouted_segments(&mut engine, default_rect()).unwrap();
            assert_eq!(segments.len(), 5);
            segments[0].clone()
        };

        // add another aggregation level
        engine
            .push(segment)
            .expect("Expect being able to push another segment");
        engine.wait().expect("Expected working wait");

        // Validate
        let segments =
            model::segmentations::layouted_segments(&mut engine, default_rect()).unwrap();
        assert_eq!(segments.len(), 2);

        // Limit to only one segment
        model::segmentations::set_segments_range(&mut engine, Some(0..=1));
        // Validate
        let segments =
            model::segmentations::layouted_segments(&mut engine, default_rect()).unwrap();
        assert_eq!(segments.len(), 1);

        // Add a filter
        let filter = Filter::Is(ValueField::bool(&Field::IsSend, true));
        model::segmentations::set_filters(&mut engine, &[filter]).expect("Expect setting filters");
        engine.wait().expect("");

        let segments =
            model::segmentations::layouted_segments(&mut engine, default_rect()).unwrap();
        assert_eq!(segments.len(), 0);

        // Remove filter
        model::segmentations::set_filters(&mut engine, &[]).expect("");
        engine.wait().expect("");

        let segments =
            model::segmentations::layouted_segments(&mut engine, default_rect()).unwrap();
        assert_eq!(segments.len(), 2);

        // Push a new segment
        let segment = segments[0].clone();
        engine.push(segment).unwrap();
        engine.wait().expect("");

        // Check the new segments
        let segments =
            model::segmentations::layouted_segments(&mut engine, default_rect()).unwrap();
        assert_eq!(segments.len(), 1);
    }
}

fn default_rect() -> Rect {
    Rect {
        left: 50.0,
        top: 50.0,
        width: 500.0,
        height: 500.0,
    }
}

fn create_database() -> Config {
    let path = "tests/resources/mbox";
    let config = Config::new(None, path, vec!["".to_string()], FormatType::Mbox).expect("Config");
    let importer = mbox_importer(config.clone());
    let database = Database::new(&config.database_path).unwrap();
    let (_receiver, handle) = importer.import(database).unwrap();
    handle.join().expect("").expect("");
    config
}
