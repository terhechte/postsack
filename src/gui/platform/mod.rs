#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::{initial_update, platform_colors, setup};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::{initial_update, platform_colors, setup};

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{initial_update, platform_colors, setup};

use eframe::egui::Color32;
/// Platform-Native Colors
#[derive(Debug)]
pub struct PlatformColors {
    pub window_background_dark: Color32,
    pub window_background_light: Color32,
    pub content_background_dark: Color32,
    pub content_background_light: Color32,
}
