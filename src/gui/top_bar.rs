use crate::cluster_engine::Engine;
use eframe::egui::{self, Widget};
use eyre::Report;

pub struct TopBar<'a> {
    engine: &'a mut Engine,
    error: &'a mut Option<Report>,
}

impl<'a> TopBar<'a> {
    pub fn new(engine: &'a mut Engine, error: &'a mut Option<Report>) -> Self {
        TopBar { engine, error }
    }
}

impl<'a> Widget for TopBar<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let groupings = self.engine.current_groupings();
            let has_back = groupings.len() > 1;
            for (id_index, group) in groupings.iter().enumerate() {
                let alternatives = self.engine.available_group_by_fields(&group);
                if let Some(value) = group.value() {
                    let label = egui::Label::new(format!("{}: {}", group.name(), value));
                    ui.add(label);
                } else if let Some(mut selected) = group.index(&alternatives) {
                    let p = egui::ComboBox::from_id_source(&id_index).show_index(
                        ui,
                        &mut selected,
                        alternatives.len(),
                        |i| alternatives[i].as_str().to_string(),
                    );
                    if p.changed() {
                        *self.error = self
                            .engine
                            .update_grouping(&group, &alternatives[selected])
                            .err();
                    }
                }
            }

            if has_back {
                if ui.button("Back").clicked() {
                    self.engine.back();
                }
            }
        })
        .response
    }
}
