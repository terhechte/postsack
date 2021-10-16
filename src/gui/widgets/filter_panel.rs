//! A panel to edit filters
use eframe::egui::{self, vec2, Response, Widget};

use crate::model::{segmentations, Engine};

/// Filter values for the UI.
/// All values are mapped as `Option<bool>`
/// as we have three states for each of them: yes, no, and all
/// Say for `is_seen1: Only seen Some(yes), Only not seen Some(no), any None
#[derive(Default)]
pub struct FilterState {
    /// Yes: All I've send, No: All I've received, None: Any
    is_send: Option<bool>,
    is_reply: Option<bool>,
    is_seen: Option<bool>,
    subject_contains: Option<String>,
    tags_contains: Option<String>,
}

impl FilterState {
    pub fn new() -> Self {
        let mut basic = Self::default();
        basic.is_send = Some(false);
        basic
    }
}

pub struct FilterPanel<'a> {
    engine: &'a mut Engine,
    state: &'a mut FilterState,
}

impl<'a> FilterPanel<'a> {
    pub fn new(engine: &'a mut Engine, state: &'a mut FilterState) -> Self {
        Self { engine, state }
    }
}

impl<'a> Widget for FilterPanel<'a> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let Self { engine, state } = self;
        egui::Frame::none()
            .margin(vec2(15.0, 10.5))
            .show(ui, |ui| {
                egui::Grid::new("filter_grid")
                    .spacing(vec2(15.0, 15.0))
                    .show(ui, |ui| {
                        // We want to have aligned labels for our input fields
                        egui::Grid::new("filter_text_grid")
                            .spacing(vec2(5.0, 15.0))
                            .show(ui, |ui| {
                                if let Some((range, total)) = segmentations::segments_range(engine)
                                {
                                    ui.label("Segment Limit");
                                    let mut selected = total;
                                    let response = ui.add(egui::Slider::new(&mut selected, range));
                                    if response.changed() {
                                        segmentations::set_segments_range(
                                            engine,
                                            Some(0..=selected),
                                        );
                                    }
                                    ui.end_row();
                                }

                                input_block(ui, "Subject", &mut state.subject_contains);
                                ui.end_row();

                                if engine.format_has_tags() {
                                    input_block(ui, "Tags", &mut state.tags_contains);
                                    ui.end_row();
                                }
                            });
                        ui.end_row();

                        radio_group(
                            ui,
                            "Send by me",
                            &["Only Send", "Only Received", "All"],
                            &mut state.is_send,
                        );
                        ui.end_row();

                        radio_group(
                            ui,
                            "Only Replies",
                            &["Yes", "No", "All"],
                            &mut state.is_reply,
                        );
                        ui.end_row();

                        if engine.format_has_seen() {
                            radio_group(ui, "Only Seen", &["Yes", "No", "All"], &mut state.is_seen);
                            ui.end_row();
                        }

                        ui.end_row();
                    })
            })
            .response
    }
}

fn radio_group(ui: &mut egui::Ui, title: &str, names: &[&str; 3], value: &mut Option<bool>) {
    let mut radio_value = match value {
        Some(true) => 0,
        Some(false) => 1,
        None => 2,
    };
    ui.vertical(|ui| {
        ui.add(egui::Label::new(title));
        //ui.end_row();
        ui.indent(&title, |ui| {
            ui.radio_value(&mut radio_value, 0, names[0]);
            ui.radio_value(&mut radio_value, 1, names[1]);
            ui.radio_value(&mut radio_value, 2, names[2]);
        });
    });

    let output_value = match radio_value {
        0 => Some(true),
        1 => Some(false),
        _ => None,
    };

    *value = output_value;
}

fn input_block(ui: &mut egui::Ui, title: &str, value: &mut Option<String>) {
    let mut text_value = value.clone().unwrap_or("".to_string());
    ui.label(title);
    ui.text_edit_singleline(&mut text_value);
    match text_value.as_str() {
        "" => *value = None,
        _ => *value = Some(text_value),
    }
}
