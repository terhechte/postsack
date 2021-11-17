use eframe::{
    egui,
    egui::{vec2, Response},
};

use super::Textures;
use super::{StateUIAction, StateUIVariant};
use crate::types::Config;

pub struct ErrorUI {
    /// The error to display
    report: eyre::Report,
    /// The config that led to this error, in order
    /// to let the user `go back`.
    /// As we might not have a config *yet* this is optional
    config: Option<Config>,
    action_back: bool,
    show_details: bool,
}

impl ErrorUI {
    pub fn new(report: eyre::Report, config: Option<Config>) -> Self {
        Self {
            report,
            config,
            action_back: false,
            show_details: false,
        }
    }
}
impl StateUIVariant for ErrorUI {
    fn update_panel(&mut self, ctx: &egui::CtxRef, _textures: &Option<Textures>) -> StateUIAction {
        egui::CentralPanel::default()
            .frame(egui::containers::Frame::group(&ctx.style()).margin(vec2(32.0, 32.0)))
            .show(ctx, |ui| {
                ui.add(|ui: &mut egui::Ui| self.ui(ui));
            });
        // If the user tapped the back button, go back to startup
        match (&self.config, self.action_back) {
            (Some(config), true) => StateUIAction::Close {
                config: config.clone(),
            },
            _ => StateUIAction::Nothing,
        }
    }
}

impl ErrorUI {
    fn ui(&mut self, ui: &mut egui::Ui) -> Response {
        let width = 250.0;

        ui.vertical_centered(|ui| {
            ui.set_width(width);
            ui.add_space(30.0);
            ui.add(
                egui::widgets::Label::new("An Error occured").text_style(egui::TextStyle::Heading),
            );
            ui.add_space(20.0);
            ui.label(format!("{}", &self.report));
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if self.config.is_some() {
                    if ui.button("Try Again").clicked() {
                        self.action_back = true;
                    }
                }
                ui.add_space(125.0);
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });
            ui.add_space(30.0);
            if ui.button("Toggle Details").clicked() {
                self.show_details = !self.show_details;
            }
            ui.add_space(30.0);
            if self.show_details {
                egui::containers::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::widgets::Label::new("Error Chain")
                            .text_style(egui::TextStyle::Heading),
                    );
                    for x in self.report.chain() {
                        ui.label(format!("{:?}", &x));
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(15.0);
                    }
                });
            }
        })
        .response
    }
}
