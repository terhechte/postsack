#![cfg(target_os = "macos")]
//! Initially, I wanted to laod the mac system fonts from
//! `/System/Library/Fonts/SFNS.ttf` but while loading these
//! worked, no actual characters were displayed.

const SYSTEM_FONT: &[u8] = include_bytes!("../fonts/mac_regular.otf");
const SYSTEM_MONO_FONT: &[u8] = include_bytes!("../fonts/mac_mono.ttc");

use cocoa;
use eframe::egui::{self, Color32, FontDefinitions, FontFamily};
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
pub fn setup(ctx: &egui::CtxRef) {
    install_fonts(ctx)
}

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

fn install_fonts(egui_ctx: &egui::CtxRef) {
    let mut fonts = FontDefinitions::default();
    for (data, family, key) in [
        (SYSTEM_FONT, FontFamily::Proportional, "Regular"),
        (SYSTEM_MONO_FONT, FontFamily::Monospace, "Mono"),
    ] {
        fonts
            .font_data
            .insert(key.to_owned(), std::borrow::Cow::Borrowed(&data));
        fonts
            .fonts_for_family
            .get_mut(&family)
            .unwrap()
            .insert(0, key.to_owned());
    }

    egui_ctx.set_fonts(fonts);
}
