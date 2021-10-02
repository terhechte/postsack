use crate::types::Config;
use eframe::{self, egui, epi};

mod app;
pub(crate) mod state;
pub(crate) mod widgets;

pub fn run_gui(config: Config) {
    let options = eframe::NativeOptions::default();
    let app: Box<dyn epi::App> = match app::MyApp::new(&config) {
        Ok(n) => Box::new(n),
        Err(e) => Box::new(ErrorApp(e)),
    };
    eframe::run_native(app, options);
}

struct ErrorApp(eyre::Report);
impl epi::App for ErrorApp {
    fn name(&self) -> &str {
        "Error"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| ui.add(widgets::ErrorBox(&self.0)));
        frame.set_window_size(ctx.used_size());
    }
}
