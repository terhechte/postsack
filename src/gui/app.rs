use eframe::epi::{Frame, Storage};
use eyre::{Report, Result};

use eframe::{egui, epi};

use super::widgets::{self, Spinner};
use crate::model::{segmentations, Engine};
use crate::types::Config;

pub struct GmailDBApp {
    _config: Config,
    engine: Engine,
    error: Option<Report>,
    show_emails: bool,
}

impl GmailDBApp {
    pub fn new(config: &Config) -> Result<Self> {
        let engine = Engine::new(&config)?;
        Ok(Self {
            _config: config.clone(),
            engine,
            error: None,
            show_emails: false,
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
        // Avoid any processing if there is an unhandled error.
        if self.error.is_none() {
            self.error = self.engine.process().err();
        }

        let Self {
            engine,
            error,
            show_emails,
            ..
        } = self;

        if let Some(error) = error {
            dbg!(&error);
            egui::CentralPanel::default().show(ctx, |ui| ui.add(widgets::ErrorBox(&error)));
        } else {
            if *show_emails {
                egui::SidePanel::right("my_left_panel")
                    .default_width(500.0)
                    .show(ctx, |ui| {
                        ui.add(super::mail_panel::MailPanel::new(engine, error));
                    });
            }

            egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
                ui.add(super::top_bar::TopBar::new(engine, error));
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                if engine.segmentations().is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.add(Spinner::new(egui::vec2(50.0, 50.0)));
                    });
                } else {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if let Some((range, total)) = segmentations::segments_range(engine) {
                                ui.label("Limit");
                                let mut selected = total;
                                let response = ui.add(egui::Slider::new(&mut selected, range));
                                if response.changed() {
                                    segmentations::set_segments_range(engine, Some(0..=selected));
                                }
                            }
                            // This is a hack to get right-alignment.
                            // we can't size the button, we can only size text. We will size text
                            // and then use ~that for the button
                            let text = "Mail";
                            let galley = ui
                                .painter()
                                .layout_no_wrap(egui::TextStyle::Button, text.to_owned());
                            ui.add_space(
                                ui.available_width()
                                    - (galley.size.x + ui.spacing().button_padding.x * 2.0),
                            );
                            if ui.add(egui::Button::new(text)).clicked() {
                                *show_emails = !*show_emails;
                            }
                        });
                        ui.add(super::widgets::Rectangles::new(engine, error));
                    });
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
