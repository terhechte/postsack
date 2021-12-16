use eframe::egui::{self, Stroke};
use eyre::{Report, Result};

use super::super::widgets::{FilterState, Spinner};
use super::Textures;
use super::{StateUIAction, StateUIVariant};
use ps_core::{model::Engine, Config};
use ps_database::Database;

#[derive(Default)]
pub struct UIState {
    pub show_emails: bool,
    pub show_filters: bool,
    pub show_export: bool,
    pub action_close: bool,
}

pub struct MainUI {
    config: Config,
    engine: Engine,
    error: Option<Report>,
    state: UIState,
    filter_state: FilterState,
    total: usize,
}

impl MainUI {
    pub fn new(config: Config, total: usize) -> Result<Self> {
        let mut engine = Engine::new::<Database>(&config)?;
        engine.start()?;
        Ok(Self {
            config,
            engine,
            error: None,
            state: UIState::default(),
            filter_state: FilterState::new(),
            total,
        })
    }
}

impl StateUIVariant for MainUI {
    fn update_panel(
        &mut self,
        ctx: &egui::CtxRef,
        _textures: &Option<Textures>,
    ) -> super::StateUIAction {
        // Avoid any processing if there is an unhandled error.
        if self.error.is_none() {
            self.error = self.engine.process().err();
        }

        let platform_colors = super::super::platform::platform_colors();

        let frame = egui::containers::Frame::none()
            .fill(platform_colors.window_background)
            .stroke(Stroke::none());

        egui::TopBottomPanel::top("panel")
            .frame(frame)
            .show(ctx, |ui| {
                ui.add(super::super::navigation_bar::NavigationBar::new(
                    &mut self.engine,
                    &mut self.error,
                    &mut self.state,
                    &mut self.filter_state,
                    self.total,
                ));
            });

        if self.state.show_emails {
            egui::SidePanel::right("left_panel")
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.add(super::super::mail_panel::MailPanel::new(
                        &mut self.engine,
                        &mut self.error,
                    ));
                });
        }

        egui::CentralPanel::default()
            .frame(egui::containers::Frame::none())
            .show(ctx, |ui| {
                if self.engine.segmentations().is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.add(Spinner::new(egui::vec2(50.0, 50.0)));
                    });
                } else {
                    let stroke = Stroke::none();
                    let fill = platform_colors.content_background;
                    super::super::widgets::background::color_background(
                        ui,
                        15.0,
                        stroke,
                        fill,
                        |ui| {
                            ui.vertical(|ui: &mut egui::Ui| {
                                ui.add(super::super::segmentation_bar::SegmentationBar::new(
                                    &mut self.engine,
                                    &mut self.error,
                                ));
                                ui.add(super::super::widgets::Rectangles::new(
                                    &mut self.engine,
                                    &mut self.error,
                                ));
                            })
                            .response
                        },
                    );
                }
            });

        // If we're waiting for a computation to succeed, we re-render again.
        if self.engine.is_busy() {
            ctx.request_repaint();
        }

        match (self.state.action_close, self.error.take()) {
            (_, Some(error)) => StateUIAction::Error {
                report: error,
                config: self.config.clone(),
            },
            (true, _) => StateUIAction::Close {
                config: self.config.clone(),
            },
            _ => StateUIAction::Nothing,
        }
    }
}
