use eframe::egui::{self, Color32, Visuals};
use once_cell::sync::OnceCell;

static INSTANCE: OnceCell<PlatformColors> = OnceCell::new();

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::{initial_update, navigation_button};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::{initial_update, navigation_button};

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{initial_update, navigation_button};

/// Platform-Native Colors
#[derive(Debug)]
pub struct PlatformColors {
    pub is_light: bool,
    pub animation_background: Color32,
    pub window_background: Color32,
    pub content_background: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    /// Brightest lines to darkest lines
    pub line1: Color32,
    pub line2: Color32,
    pub line3: Color32,
    pub line4: Color32,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn visuals(&self) -> Visuals {
        match self {
            Theme::Light => egui::Visuals::light(),
            Theme::Dark => egui::Visuals::dark(),
        }
    }
}

pub fn setup(ctx: &egui::CtxRef, theme: Theme) {
    #[cfg(target_os = "windows")]
    use windows as module;

    #[cfg(target_os = "linux")]
    use linux as module;

    #[cfg(target_os = "macos")]
    use macos as module;

    INSTANCE
        .set(module::platform_colors(theme))
        .expect("Could not setup colors");
    let colors = module::platform_colors(theme);
    let mut visuals = theme.visuals();
    visuals.widgets.noninteractive.bg_fill = colors.window_background;
    ctx.set_visuals(visuals);
    module::setup(ctx)
}

pub fn platform_colors() -> &'static PlatformColors {
    INSTANCE.get().unwrap()
}
