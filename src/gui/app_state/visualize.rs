use eframe::egui::{self, Response, Stroke, Widget};
use eyre::Report;

use super::super::widgets::{self, FilterState, Spinner};

use crate::model::Engine;

#[derive(Default)]
pub struct UIState {
    pub show_emails: bool,
    pub show_filters: bool,
    pub show_export: bool,
    pub action_close: bool,
}

pub struct Visualize {
    engine: Engine,
    error: Option<Report>,
    state: UIState,
    filter_state: FilterState,
    platform_custom_setup: bool,
}

impl Widget for &mut Visualize {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        // Avoid any processing if there is an unhandled error.
        if self.error.is_none() {
            self.error = self.engine.process().err();
        }

        if !self.platform_custom_setup {
            self.platform_custom_setup = true;
            self.error = super::super::platform::initial_update(&ui.ctx()).err();

            // Make the UI a bit bigger
            let pixels = ui.ctx().pixels_per_point();
            ui.ctx().set_pixels_per_point(pixels * 1.2)
        }

        let platform_colors = super::super::platform::platform_colors();

        let response = if let Some(error) = self.error.as_ref() {
            dbg!(&error);
            egui::CentralPanel::default()
                .show(ui.ctx(), |ui| ui.add(widgets::ErrorBox(error)))
                .response
        } else {
            let frame = egui::containers::Frame::none()
                .fill(platform_colors.window_background_dark)
                .stroke(Stroke::none());

            egui::TopBottomPanel::top("my_panel")
                .frame(frame)
                .show(ui.ctx(), |ui| {
                    ui.add(super::super::navigation_bar::NavigationBar::new(
                        &mut self.engine,
                        &mut self.error,
                        &mut self.state,
                        &mut self.filter_state,
                    ));
                });

            if self.state.show_emails {
                egui::SidePanel::right("my_left_panel")
                    .default_width(500.0)
                    .show(ui.ctx(), |ui| {
                        ui.add(super::super::mail_panel::MailPanel::new(
                            &mut self.engine,
                            &mut self.error,
                        ));
                    });
            }

            egui::CentralPanel::default()
                .frame(egui::containers::Frame::none())
                .show(ui.ctx(), |ui| {
                    if self.engine.segmentations().is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.add(Spinner::new(egui::vec2(50.0, 50.0)));
                        });
                    } else {
                        let stroke = Stroke::none();
                        let fill = platform_colors.content_background_dark;
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
                })
                .response
        };

        // If we're waiting for a computation to succeed, we re-render again.
        if self.engine.is_busy() {
            ui.ctx().request_repaint();
        }

        response
    }
}
