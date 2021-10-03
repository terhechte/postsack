use eframe::epi::{Frame, Storage};
use eyre::{Report, Result};

use eframe::{egui, epi};

use super::widgets::{self, Spinner};
use crate::cluster_engine::Engine;
use crate::types::Config;

pub struct GmailDBApp {
    _config: Config,
    engine: Engine,
    error: Option<Report>,
}

impl GmailDBApp {
    pub fn new(config: &Config) -> Result<Self> {
        let engine = Engine::new(&config)?;
        Ok(Self {
            _config: config.clone(),
            engine,
            error: None,
        })
    }
}

impl epi::App for GmailDBApp {
    fn name(&self) -> &str {
        "Gmail DB"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut Frame<'_>,
        _storage: Option<&dyn Storage>,
    ) {
        self.error = self.engine.start().err();
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        self.error = self.engine.process().err();

        let Self { engine, error, .. } = self;

        if let Some(error) = error {
            egui::CentralPanel::default().show(ctx, |ui| ui.add(widgets::ErrorBox(&error)));
        } else {
            egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
                ui.heading("GMail DB");
                ui.horizontal(|ui| {
                    ui.label("Search");
                });
            });

            egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
                ui.add(super::widgets::TopBar::new(engine, error));
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                if engine.is_busy() {
                    ui.centered_and_justified(|ui| {
                        ui.add(Spinner::new(egui::vec2(50.0, 50.0)));
                    });
                } else {
                    ui.add(super::widgets::Rectangles::new(engine, error));
                }
            });
        }

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());

        // If we're waiting for a computation to succeed, we re-render again.
        // The initial plan of calling `ctx.request_repaint()` from a thread didn't work.
        if engine.is_busy() {
            ctx.request_repaint();
        }
    }
}
