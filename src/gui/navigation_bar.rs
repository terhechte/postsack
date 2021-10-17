use crate::model::Engine;
use eframe::egui::{self, Widget};
use eyre::Report;

use super::app_state::UIState;
use super::platform::navigation_button;
use super::widgets::{FilterPanel, FilterState};

pub struct NavigationBar<'a> {
    engine: &'a mut Engine,
    #[allow(unused)]
    error: &'a mut Option<Report>,
    state: &'a mut UIState,
    filter_state: &'a mut FilterState,
}

impl<'a> NavigationBar<'a> {
    pub fn new(
        engine: &'a mut Engine,
        error: &'a mut Option<Report>,
        state: &'a mut UIState,
        filter_state: &'a mut FilterState,
    ) -> Self {
        NavigationBar {
            engine,
            error,
            state,
            filter_state,
        }
    }
}

impl<'a> Widget for NavigationBar<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Override the button spacing
        ui.spacing_mut().button_padding = (6.0, 3.0).into();

        ui.visuals_mut().widgets.inactive.corner_radius = 5.0;
        ui.visuals_mut().widgets.active.corner_radius = 5.0;
        ui.visuals_mut().widgets.hovered.corner_radius = 5.0;

        let response = ui
            .horizontal(|ui| {
                ui.set_height(40.0);

                ui.add_space(15.0);

                let close_text = "\u{23F4} Close";
                if ui.add(navigation_button(close_text)).clicked() {
                    self.state.action_close = true;
                }

                let filter_text = "\u{1f50D} Filters";
                let filter_response = ui.add(navigation_button(filter_text));
                let popup_id = ui.make_persistent_id("filter_panel_id");

                super::widgets::popover(ui, popup_id, &filter_response, |ui| {
                    ui.add(FilterPanel::new(self.engine, self.filter_state));
                });

                // This is a hack to get right-alignment.
                // we can't size the button, we can only size text. We will size text
                // and then use ~that for these buttons
                let mut w = ui.available_width();

                let mail_text = "\u{1F4E7} Mails";
                let mail_galley = ui
                    .painter()
                    .layout_no_wrap(egui::TextStyle::Button, mail_text.to_owned());

                let filter_text = "\u{1F5B9} Export";
                let filter_galley = ui
                    .painter()
                    .layout_no_wrap(egui::TextStyle::Button, filter_text.to_owned());

                w -= mail_galley.size.x + ui.spacing().button_padding.x * 4.0;
                w -= filter_galley.size.x + ui.spacing().button_padding.x * 4.0;
                ui.add_space(w);

                ui.add(navigation_button(filter_text));

                if ui.add(navigation_button(mail_text)).clicked() {
                    self.state.show_emails = !self.state.show_emails;
                }
            })
            .response;
        response
    }
}
