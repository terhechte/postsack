use eframe::egui;
use postsack::{
    self,
    database::query::{Field, Filter, ValueField},
    importer::Importerlike,
    model::{self, Engine},
    types::Config,
    types::FormatType,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_all() {
        let config = create_database();
        let mut engine = Engine::new(&config).expect("Expected Engine");
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

fn default_rect() -> egui::Rect {
    egui::Rect::from_min_size(
        egui::Pos2 { x: 50.0, y: 50.0 },
        egui::Vec2 { x: 500.0, y: 500.0 },
    )
}

fn create_database() -> Config {
    let path = "tests/resources/mbox";
    let config = postsack::types::Config::new(None, path, vec!["".to_string()], FormatType::Mbox)
        .expect("Config");
    let importer = postsack::importer::mbox_importer(config.clone());
    let (_receiver, handle) = importer.import().unwrap();
    handle.join().expect("").expect("");
    config
}
