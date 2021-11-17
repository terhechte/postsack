use crate::model::{segmentations, Engine};
use eframe::egui::{self, Widget};
use eyre::Report;

pub struct SegmentationBar<'a> {
    engine: &'a mut Engine,
    error: &'a mut Option<Report>,
}

impl<'a> SegmentationBar<'a> {
    pub fn new(engine: &'a mut Engine, error: &'a mut Option<Report>) -> Self {
        Self { engine, error }
    }
}

impl<'a> Widget for SegmentationBar<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.set_height(30.0);

            ui.label("Group By:");

            let groupings = segmentations::aggregated_by(self.engine);
            let has_back = groupings.len() > 1;
            for (id_index, group) in groupings.iter().enumerate() {
                let alternatives = segmentations::aggregation_fields(self.engine, group);
                if let Some(value) = group.value() {
                    ui.add_enabled(
                        false,
                        egui::Button::new(format!("{} {}", group.name(), value))
                            .text_color(egui::Color32::WHITE),
                    );
                } else if let Some(mut selected) = group.index(&alternatives) {
                    let p = egui::ComboBox::from_id_source(&id_index).show_index(
                        ui,
                        &mut selected,
                        alternatives.len(),
                        |i| alternatives[i].name().to_string(),
                    );
                    if p.changed() {
                        *self.error = segmentations::set_aggregation(
                            self.engine,
                            group,
                            &alternatives[selected],
                        )
                        .err();
                    }
                }
            }

            if has_back && ui.button("\u{2716}").clicked() {
                self.engine.pop();
            }
        })
        .response
    }
}
