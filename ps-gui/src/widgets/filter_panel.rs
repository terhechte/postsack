//! A panel to edit filters
use eframe::egui::{self, vec2, Color32, Response, Widget};
use eyre::Report;

use ps_core::{
    model::{segmentations, Engine},
    Field, Filter, ValueField,
};

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
        FilterState {
            is_send: Some(false),
            ..Default::default()
        }
    }

    fn apply(&self, engine: &mut Engine, error: &mut Option<Report>) {
        // FIXME: In principle this could rather be logic for the `engine`, but I'd like to have a generic engine at some point.
        let mut filters = Vec::new();
        if let Some(val) = self.is_send {
            filters.push(Filter::Is(ValueField::bool(&Field::IsSend, val)));
        }
        if let Some(val) = self.is_seen {
            filters.push(Filter::Is(ValueField::bool(&Field::MetaIsSeen, val)));
        }
        if let Some(val) = self.is_reply {
            filters.push(Filter::Is(ValueField::bool(&Field::IsReply, val)));
        }
        // FIXME: The system currently doesn't allow searching for multiple tags
        // (e.g. (x like tag1 or x like tag2))
        // this would require a `Filter::Expression` that is just added verbatim
        if let Some(n) = &self.tags_contains {
            filters.push(Filter::Like(ValueField::string(
                &Field::MetaTags,
                n.clone(),
            )));
        }
        if let Some(n) = &self.subject_contains {
            filters.push(Filter::Contains(ValueField::string(
                &Field::Subject,
                n.clone(),
            )));
        }
        *error = segmentations::set_filters(engine, &filters).err();
    }

    fn clear(&mut self) {
        self.is_send = None;
        self.is_reply = None;
        self.is_seen = None;
        self.subject_contains = None;
        self.tags_contains = None;
    }
}

pub struct FilterPanel<'a> {
    engine: &'a mut Engine,
    state: &'a mut FilterState,
    error: &'a mut Option<Report>,
}

impl<'a> FilterPanel<'a> {
    pub fn new(
        engine: &'a mut Engine,
        state: &'a mut FilterState,
        error: &'a mut Option<Report>,
    ) -> Self {
        Self {
            engine,
            state,
            error,
        }
    }
}

impl<'a> Widget for FilterPanel<'a> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let Self {
            engine,
            state,
            error,
        } = self;
        egui::Frame::none()
            .margin(vec2(15.0, 10.5))
            .show(ui, |ui| {
                FilterPanel::filter_panel_contents(ui, engine, state);
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        state.apply(engine, error);
                    }
                    ui.add_space(10.0);
                    if ui.button("Clear").clicked() {
                        state.clear();
                        state.apply(engine, error);
                    }
                });
            })
            .response
    }
}

impl FilterPanel<'_> {
    fn filter_panel_contents(
        ui: &mut egui::Ui,
        engine: &mut Engine,
        state: &mut FilterState,
    ) -> Response {
        egui::ScrollArea::vertical()
            .max_height(330.0)
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
                            });
                        ui.end_row();

                        if engine.format_has_tags() {
                            input_tags(
                                ui,
                                "Labels / Tags",
                                &mut state.tags_contains,
                                engine.known_tags(),
                            );
                            ui.end_row();
                        }

                        radio_group(
                            ui,
                            "Inbox",
                            &["Only Send Mails", "Only Received Mails", "All"],
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
    let mut text_value = value.clone().unwrap_or_else(|| "".to_string());
    ui.label(title);
    ui.text_edit_singleline(&mut text_value);
    match text_value.as_str() {
        "" => *value = None,
        _ => *value = Some(text_value),
    }
}

fn input_tags(
    ui: &mut egui::Ui,
    title: &str,
    selection: &mut Option<String>,
    available: &[String],
) {
    ui.vertical(|ui| {
        ui.add(egui::Label::new(title));
        egui::Frame::none()
            .margin((5.0, 5.0))
            .corner_radius(8.0)
            .fill(Color32::BLACK)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    // Overly complicated, but can later on easily be extended for multi selection
                    for (index, tag) in available.iter().enumerate() {
                        let was_selected = if let Some(n) = selection {
                            n == tag
                        } else {
                            false
                        };
                        let mut selected = if was_selected { index } else { 9999 };
                        if ui.selectable_value(&mut selected, index, tag).clicked() {
                            if was_selected {
                                *selection = None;
                            } else {
                                *selection = Some(tag.clone());
                            }
                        }
                    }
                });
            });
        // if ui.button("Clear").clicked() {
        //     selection.clear();
        // }
    });
}
