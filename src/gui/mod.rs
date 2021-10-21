use eframe::{self, egui, epi};

mod app;
mod app_state;
mod mail_panel;
mod navigation_bar;
mod platform;
mod segmentation_bar;
pub(crate) mod widgets;

pub fn run_gui() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app::GmailDBApp::new()), options);
}
