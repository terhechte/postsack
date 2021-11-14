#![cfg(target_os = "windows")]

use eframe::egui;

use super::PlatformColors;

pub fn platform_colors() -> PlatformColors {
    // From Google images, Windows 11
    PlatformColors {
        window_background_dark: Color32::from_rgb(32, 32, 32),
        window_background_light: Color32::from_rgb(241, 243, 246),
        content_background_dark: Color32::from_rgb(34, 32, 40),
        content_background_light: Color32::from_rgb(251, 251, 253),
    }
}

/// This is called from `App::setup`
pub fn setup(ctx: &egui::CtxRef) {}

/// This is called once from `App::update` on the first run.
pub fn initial_update(ctx: &egui::CtxRef) -> Result<()> {}

pub fn navigation_button(title: &str) -> egui::Button {
    egui::Button::new(title)
}
