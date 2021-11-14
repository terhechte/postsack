#![cfg(target_os = "windows")]

use eframe::egui::{self, Color32};

use super::{PlatformColors, Theme};

pub fn platform_colors() -> PlatformColors {
    // From Google images, Windows 11
    match theme {
        Theme::Light => PlatformColors {
            window_background: Color32::from_rgb(241, 243, 246),
            content_background: Color32::from_rgb(251, 251, 253),
        },
        Theme::Dark => PlatformColors {
            window_background: Color32::from_rgb(32, 32, 32),
            content_background: Color32::from_rgb(34, 32, 40),
        },
    }
}

/// This is called from `App::setup`
pub fn setup(ctx: &egui::CtxRef) {}

/// This is called once from `App::update` on the first run.
pub fn initial_update(ctx: &egui::CtxRef) -> Result<()> {}

pub fn navigation_button(title: &str) -> egui::Button {
    egui::Button::new(title)
}
