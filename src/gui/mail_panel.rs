use crate::cluster_engine::Engine;
use crate::database::query::Field;
use eframe::egui::{self, Widget};
use eyre::Report;

use super::widgets::Table;

pub struct MailPanel<'a> {
    engine: &'a mut Engine,
    error: &'a mut Option<Report>,
}

impl<'a> MailPanel<'a> {
    pub fn new(engine: &'a mut Engine, error: &'a mut Option<Report>) -> Self {
        MailPanel { engine, error }
    }
}
impl<'a> Widget for MailPanel<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let empty_vec = Vec::new();
        let mut selected_row: Option<usize> = None;
        ui.vertical(|ui| {
            ui.add(
                Table::new_selectable(
                    "mail_list",
                    &mut selected_row,
                    self.engine.current_element_count(),
                    |range| {
                        // we overshoot the range a bit, as otherwise somehow the bottom is always empty
                        let range = std::ops::Range {
                            start: range.start,
                            end: range.end + 6,
                        };
                        let rows = match self.engine.current_contents(&range) {
                            Ok(n) => n,
                            Err(e) => {
                                *self.error = Some(e);
                                empty_vec.clone()
                            }
                        };
                        rows
                    },
                )
                .column("Sender", 130.0, |sample| {
                    let sample = match sample {
                        Some(n) => n,
                        None => return "".to_owned(),
                    };
                    format!(
                        "{}@{}",
                        sample[&Field::SenderLocalPart].value().as_str().unwrap(),
                        sample[&Field::SenderDomain].value().as_str().unwrap()
                    )
                })
                .column("Subject", 400.0, |sample| {
                    let sample = match sample {
                        Some(n) => n,
                        None => return "".to_owned(),
                    };
                    sample[&Field::Subject]
                        .value()
                        .as_str()
                        .unwrap()
                        .to_string()
                }),
            )
        })
        .response
    }
}
