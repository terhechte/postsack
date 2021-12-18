#![cfg(target_os = "macos")]
//! Initially, I wanted to laod the mac system fonts from
//! `/System/Library/Fonts/SFNS.ttf` but while loading these
//! worked, no actual characters were displayed.

const SYSTEM_FONT: &[u8] = include_bytes!("../fonts/mac_regular.otf");
const SYSTEM_MONO_FONT: &[u8] = include_bytes!("../fonts/mac_mono.ttc");

use cocoa;
use eframe::egui::{self, Color32, FontDefinitions, FontFamily, Stroke};
use ps_core::eyre::{bail, Result};

use objc::runtime::{Object, YES};
use objc::*;

use super::{PlatformColors, Theme};

pub fn platform_colors(theme: Theme) -> PlatformColors {
    match theme {
        Theme::Light => PlatformColors {
            is_light: true,
            animation_background: Color32::from_rgb(248, 246, 249),
            window_background: Color32::from_rgb(238, 236, 242),
            content_background: Color32::from_rgb(236, 234, 238),
            text_primary: Color32::from_gray(0),
            text_secondary: Color32::from_gray(30),
            line1: Color32::from_gray(0),
            line2: Color32::from_gray(30),
            line3: Color32::from_gray(60),
            line4: Color32::from_gray(90),
        },
        Theme::Dark => PlatformColors {
            is_light: false,
            animation_background: Color32::from_rgb(0, 0, 0),
            window_background: Color32::from_rgb(36, 30, 42),
            content_background: Color32::from_rgb(20, 14, 26),
            text_primary: Color32::from_gray(255),
            text_secondary: Color32::from_gray(200),
            line1: Color32::from_gray(255),
            line2: Color32::from_gray(190),
            line3: Color32::from_gray(150),
            line4: Color32::from_gray(70),
        },
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

/// Return a primary button intended for the top navigation in the
/// platform style.
/// FIXME: Dark / Light Mode distinction!
pub fn navigation_button(title: &str) -> egui::Button {
    // Finder info panel bottom button.
    let fill = Color32::from_rgb(87, 84, 92);
    // let stroke = Color32::from_rgb(97, 94, 102);
    let text = Color32::WHITE;
    egui::Button::new(title)
        .text_color(text)
        .stroke(Stroke::new(1.0, fill))
        .fill(fill)
}

fn install_fonts(egui_ctx: &egui::CtxRef) {
    let mut fonts = FontDefinitions::default();
    for (data, family, key) in [
        (SYSTEM_FONT, FontFamily::Proportional, "Regular"),
        (SYSTEM_MONO_FONT, FontFamily::Monospace, "Mono"),
    ] {
        fonts
            .font_data
            .insert(key.to_owned(), std::borrow::Cow::Borrowed(data));
        fonts
            .fonts_for_family
            .get_mut(&family)
            .unwrap()
            .insert(0, key.to_owned());
    }

    egui_ctx.set_fonts(fonts);
}
