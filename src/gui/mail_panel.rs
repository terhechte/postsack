use crate::cluster_engine::Engine;
use crate::database::query::{Field, Value};
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
                    |range| match self.engine.current_contents(&range) {
                        Ok(Some(n)) => n.clone(),
                        Ok(None) => {
                            *self.error = self.engine.request_contents(&range).err();
                            empty_vec.clone()
                        }
                        Err(e) => {
                            *self.error = Some(e);
                            empty_vec.clone()
                        }
                    },
                )
                .column("Sender", |sample| {
                    format!(
                        "{}@{}",
                        sample[&Field::SenderLocalPart].value().as_str().unwrap(),
                        sample[&Field::SenderDomain].value().as_str().unwrap()
                    )
                })
                .column("Subject", |sample| {
                    sample[&Field::Subject].value().to_string()
                }),
            )
        })
        .response
    }
}
