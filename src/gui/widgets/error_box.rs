use eframe::{self, egui};

pub struct ErrorBox<'a>(pub &'a eyre::Report);

impl<'a> egui::Widget for ErrorBox<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let text = format!("Error:\n{}", &self.0);
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add(eframe::egui::Label::new(text));
                if ui.button("Close").clicked() {
                    std::process::exit(0);
                }
            });
        })
        .response
    }
}
