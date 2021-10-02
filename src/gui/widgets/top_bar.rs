use crate::cluster_engine::Engine;
use eframe::egui;

fn top_bar_ui(ui: &mut egui::Ui, engine: &mut Engine) -> egui::Response {
    ui.horizontal(|ui| {
        let groupings = engine.current_groupings();
        let has_back = groupings.len() > 1;
        for (index, group_index, value) in groupings {
            if let Some(value) = value {
                let label = egui::Label::new(format!(
                    "{}: {}",
                    &value.as_group_field().as_str(),
                    value.value()
                ));
                ui.add(label);
            } else {
                let alternatives = Engine::all_group_by_fields();
                let mut selected = group_index;
                let p = egui::ComboBox::from_id_source(&index).show_index(
                    ui,
                    &mut selected,
                    alternatives.len(),
                    |i| alternatives[i].as_str().to_string(),
                );
                if p.changed() {
                    engine.update_grouping(index, selected);
                }
            }
        }

        if has_back {
            if ui.button("Back").clicked() {
                engine.back();
            }
        }
    })
    .response
}

pub fn top_bar(engine: &mut Engine) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| top_bar_ui(ui, engine)
}
