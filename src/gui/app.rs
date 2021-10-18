use eframe::{
    egui::{self},
    epi::{self, Frame, Storage},
};
use eyre::Result;

use super::app_state::{self, Import, Startup, Visualize};

pub enum GmailDBApp {
    Startup { panel: Startup },
    Import { panel: Import },
    Visualize { panel: Visualize },
}

impl GmailDBApp {
    pub fn new() -> Result<Self> {
        // Temporarily create config without state machine
        let config = app_state::make_temporary_ui_config();
        Ok(GmailDBApp::Import {
            panel: Import::new(config),
        })
    }
}

impl epi::App for GmailDBApp {
    fn name(&self) -> &str {
        "Gmail DB"
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &mut Frame<'_>,
        _storage: Option<&dyn Storage>,
    ) {
        // FIXME: Bring back
        //self.error = self.engine.start().err();
        super::platform::setup(ctx);

        // Adapt to the platform colors
        let platform_colors = super::platform::platform_colors();
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = platform_colors.window_background_dark;
        ctx.set_visuals(visuals);

        // Make the UI a bit bigger
        let pixels = ctx.pixels_per_point();
        ctx.set_pixels_per_point(pixels * 1.2)
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        match self {
            GmailDBApp::Startup { panel } => Self::update_panel(panel, ctx, frame),
            GmailDBApp::Import { panel } => Self::update_panel(panel, ctx, frame),
            _ => panic!(),
        }

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}

impl GmailDBApp {
    fn update_panel(panel: impl egui::Widget, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default()
            .frame(egui::containers::Frame::none())
            .show(ctx, |ui| {
                ui.add(panel);
            });
    }
}
