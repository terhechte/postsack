use eframe::{
    egui,
    egui::{Response, Widget},
};

use super::super::widgets;
use super::{StateUI, StateUIAction, StateUIVariant};
use crate::types::Config;

pub struct ErrorUI {
    /// The error to display
    report: eyre::Report,
    /// The config that led to this error, in order
    /// to let the user `go back`.
    /// As we might not have a config *yet* this is optional
    config: Option<Config>,
}

impl ErrorUI {
    pub fn new(report: eyre::Report, config: Option<Config>) -> Self {
        Self { report, config }
    }
}
impl StateUIVariant for ErrorUI {
    fn update_panel(&mut self, ctx: &egui::CtxRef) -> StateUIAction {
        egui::CentralPanel::default()
            .frame(egui::containers::Frame::none())
            .show(ctx, |ui| {
                ui.add(|ui: &mut egui::Ui| self.ui(ui));
            });
        // If the user tapped the back button, go back to startup
        StateUIAction::Nothing
    }
}

impl ErrorUI {
    fn ui(&mut self, ui: &mut egui::Ui) -> Response {
        // FIXME: Try again button
        // that goes back to the `Startup` screen.
        // somehow it should also fill it out again?
        ui.add(widgets::ErrorBox(&self.report))
    }
}
