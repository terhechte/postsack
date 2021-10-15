#![cfg(target_os = "macos")]

use cocoa;
use eframe::egui::{self, Color32};
use eyre::{bail, Result};
use objc::runtime::{Object, YES};

use super::PlatformColors;

pub fn platform_colors() -> PlatformColors {
    PlatformColors {
        window_background_dark: Color32::from_rgb(36, 30, 42),
        window_background_light: Color32::from_rgb(238, 236, 242),
        content_background_dark: Color32::from_rgb(20, 14, 26),
        content_background_light: Color32::from_rgb(236, 234, 238),
    }
}

/// This is called from `App::setup`
pub fn setup(_ctx: &egui::CtxRef) {}

/// This is called once from `App::update` on the first run.
pub fn initial_update(_ctx: &egui::CtxRef) -> Result<()> {
    unsafe {
        let app = cocoa::appkit::NSApp();
        if app.is_null() {
            bail!("Could not retrieve NSApp");
        }
        let main_window: *mut Object = msg_send![app, mainWindow];

        let _: () = msg_send![main_window, setTitlebarAppearsTransparent: YES];
    }
    Ok(())
}
