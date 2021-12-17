mod app;
mod app_state;
mod mail_panel;
mod navigation_bar;
mod platform;
mod segmentation_bar;
mod textures;
pub(crate) mod widgets;

use ps_core::DatabaseLike;

pub fn run_ui<Database: DatabaseLike>() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app::PostsackApp::<Database>::new()), options);
}
