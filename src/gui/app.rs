use eframe::{
    egui::{self},
    epi::{self, App, Frame, Storage},
};

use super::app_state::StateUI;

pub struct GmailDBApp {
    state: StateUI,
    platform_custom_setup: bool,
}

impl GmailDBApp {
    pub fn new() -> Self {
        let state = StateUI::new();
        GmailDBApp {
            state,
            platform_custom_setup: false,
        }
    }
}

impl App for GmailDBApp {
    fn name(&self) -> &str {
        "Gmail DB"
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &mut Frame<'_>,
        _storage: Option<&dyn Storage>,
    ) {
        super::platform::setup(ctx);

        // Adapt to the platform colors
        let platform_colors = super::platform::platform_colors();
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = platform_colors.window_background_dark;
        ctx.set_visuals(visuals);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        if !self.platform_custom_setup {
            self.platform_custom_setup = true;

            // Make the UI a bit bigger
            let pixels = ctx.pixels_per_point();
            ctx.set_pixels_per_point(pixels * 1.2);

            // If there is a platform error, display it
            if let Some(e) = super::platform::initial_update(&ctx).err() {
                self.state = StateUI::error(e);
            }
        }

        self.state.update(ctx);

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}
