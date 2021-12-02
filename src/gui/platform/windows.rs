#![cfg(target_os = "windows")]

use eframe::egui::{self, Color32};
use eyre::Result;

use super::{PlatformColors, Theme};

pub fn platform_colors(theme: Theme) -> PlatformColors {
    // From Google images, Windows 11
    match theme {
        Theme::Light => PlatformColors {
            is_light: true,
            animation_background: Color32::from_rgb(248, 246, 249),
            window_background: Color32::from_rgb(241, 243, 246),
            content_background: Color32::from_rgb(251, 251, 253),
            text_primary: Color32::from_gray(0),
            text_secondary: Color32::from_gray(30),
            line1: Color32::from_gray(0),
            line2: Color32::from_gray(30),
            line3: Color32::from_gray(60),
            line4: Color32::from_gray(90),
        },
        Theme::Dark => PlatformColors {
            is_light: false,
            animation_background: Color32::from_gray(60),
            window_background: Color32::from_rgb(32, 32, 32),
            content_background: Color32::from_rgb(34, 32, 40),
            text_primary: Color32::from_gray(255),
            text_secondary: Color32::from_gray(200),
            line1: Color32::from_gray(255),
            line2: Color32::from_gray(210),
            line3: Color32::from_gray(190),
            line4: Color32::from_gray(120),
        },
    }
}

/// This is called from `App::setup`
pub fn setup(ctx: &egui::CtxRef) {}

/// This is called once from `App::update` on the first run.
pub fn initial_update(ctx: &egui::CtxRef) -> Result<()> {
    Ok(())
}

pub fn navigation_button(title: &str) -> egui::Button {
    egui::Button::new(title)
}
