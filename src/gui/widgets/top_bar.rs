use crate::cluster_engine::Engine;
use eframe::egui;

fn top_bar_ui(ui: &mut egui::Ui, engine: &mut Engine) -> egui::Response {
    ui.horizontal(|ui| {
        let groupings = engine.current_groupings();
        let has_back = groupings.len() > 1;
        for (id_index, group) in groupings.iter().enumerate() {
            let alternatives = engine.available_group_by_fields(&group);
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
                    engine.update_grouping(&group, &alternatives[selected]);
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
