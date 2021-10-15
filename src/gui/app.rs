use eframe::{
    egui::{self, Stroke},
    epi::{self, Frame, Storage},
};
use eyre::{Report, Result};

use super::widgets::{self, Spinner};
use crate::model::Engine;
use crate::types::Config;

#[derive(Default)]
pub struct UIState {
    pub show_emails: bool,
    pub show_filters: bool,
    pub show_export: bool,
    pub action_close: bool,
}

pub struct GmailDBApp {
    _config: Config,
    engine: Engine,
    error: Option<Report>,
    state: UIState,
    platform_custom_setup: bool,
}

impl GmailDBApp {
    pub fn new(config: &Config) -> Result<Self> {
        let engine = Engine::new(config)?;
        Ok(Self {
            _config: config.clone(),
            engine,
            error: None,
            state: UIState::default(),
            platform_custom_setup: false,
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
        self.error = self.engine.start().err();
        super::platform::setup(ctx);

        // Adapt to the platform colors
        let platform_colors = super::platform::platform_colors();
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = platform_colors.window_background_dark;
        ctx.set_visuals(visuals);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        // Avoid any processing if there is an unhandled error.
        if self.error.is_none() {
            self.error = self.engine.process().err();
        }

        if !self.platform_custom_setup {
            self.platform_custom_setup = true;
            self.error = super::platform::initial_update(&ctx).err();
        }

        let Self {
            engine,
            error,
            state,
            ..
        } = self;

        let platform_colors = super::platform::platform_colors();

        if let Some(error) = error {
            dbg!(&error);
            egui::CentralPanel::default().show(ctx, |ui| ui.add(widgets::ErrorBox(error)));
        } else {
            let frame = egui::containers::Frame::none()
                .fill(platform_colors.window_background_dark)
                .stroke(Stroke::none());
            egui::TopBottomPanel::top("my_panel")
                .frame(frame)
                .show(ctx, |ui| {
                    ui.add(super::top_bar::TopBar::new(engine, error, state));
                });

            if state.show_emails {
                egui::SidePanel::right("my_left_panel")
                    .default_width(500.0)
                    .show(ctx, |ui| {
                        ui.add(super::mail_panel::MailPanel::new(engine, error));
                    });
            }

            egui::CentralPanel::default()
                .frame(egui::containers::Frame::none())
                .show(ctx, |ui| {
                    if engine.segmentations().is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.add(Spinner::new(egui::vec2(50.0, 50.0)));
                        });
                    } else {
                        let stroke = Stroke::none();
                        let fill = platform_colors.content_background_dark;
                        super::widgets::background::background_color(ui, 15.0, stroke, fill, |ui| {
                            ui.vertical(|ui: &mut egui::Ui| {
                                ui.add(super::segmentation_bar::SegmentationBar::new(
                                    engine, error,
                                ));
                                ui.add(super::widgets::Rectangles::new(engine, error));
                            });
                        })
                    }
                });
        }

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());

        // If we're waiting for a computation to succeed, we re-render again.
        if engine.is_busy() {
            ctx.request_repaint();
        }
    }
}
