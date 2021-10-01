use crate::types::Config;

mod app;
pub(crate) mod state;
pub(crate) mod widgets;

pub fn run_gui(config: Config) {
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app::MyApp::new(&config)), options);
}
