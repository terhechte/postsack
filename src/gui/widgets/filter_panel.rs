//! A panel to edit filters
use eframe::egui::{self, vec2, Response, TextStyle, Vec2, Widget};

use crate::model::{segmentations, Engine};

pub struct FilterPanel<'a> {
    engine: &'a mut Engine,
}

impl<'a> FilterPanel<'a> {
    pub fn default_size() -> Vec2 {
        vec2(200.0, 400.0)
    }
    pub fn new(engine: &'a mut Engine) -> Self {
        Self { engine }
    }
}

impl<'a> Widget for FilterPanel<'a> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let Self { engine } = self;
        ui.vertical(|ui| {
            if let Some((range, total)) = segmentations::segments_range(engine) {
                ui.horizontal(|ui| {
                    ui.label("Limit");
                    let mut selected = total;
                    let response = ui.add(egui::Slider::new(&mut selected, range));
                    if response.changed() {
                        segmentations::set_segments_range(engine, Some(0..=selected));
                    }
                });
            }
            ui.label("label");
            ui.label("label");
            ui.label("label");
            ui.label("label");
            ui.label("label");
            ui.label("label");
            ui.label("label");
        })
        .response
    }
}
