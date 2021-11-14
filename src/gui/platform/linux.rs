#![cfg(target_os = "linux")]

use eframe::egui::{self, Color32};
use eyre::Result;

use super::{PlatformColors, Theme};

pub fn platform_colors(theme: Theme) -> PlatformColors {
    // From Google images, Gtk
    match theme {
        Theme::Light => PlatformColors {
            window_background: Color32::from_rgb(246, 245, 244),
            content_background: Color32::from_rgb(254, 254, 254),
        },
        Theme::Dark => PlatformColors {
            window_background: Color32::from_rgb(73, 73, 73),
            content_background: Color32::from_rgb(34, 32, 40),
        },
    }
}

/// This is called from `App::setup`
pub fn setup(_ctx: &egui::CtxRef) {}

/// This is called once from `App::update` on the first run.
pub fn initial_update(_ctx: &egui::CtxRef) -> Result<()> {
    Ok(())
}

pub fn navigation_button(title: &str) -> egui::Button {
    egui::Button::new(title)
}
