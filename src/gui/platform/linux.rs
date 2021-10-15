#![cfg(target_os = "linux")]

use eframe::egui;

use super::PlatformColors;

pub fn platform_colors() -> PlatformColors {
    // From Google images, Gtk
    PlatformColors {
        window_background_dark: Color32::from_rgb(53, 53, 53),
        window_background_light: Color32::from_rgb(246, 245, 244),
        content_background_dark: Color32::from_rgb(34, 32, 40),
        content_background_light: Color32::from_rgb(254, 254, 254),
    }
}

/// This is called from `App::setup`
pub fn setup(ctx: &egui::CtxRef) {}

/// This is called once from `App::update` on the first run.
pub fn initial_update(ctx: &egui::CtxRef) -> Result<()> {}
