use eframe::egui::{self, Color32, Label, Widget};
use num_format::{Locale, ToFormattedString};
use ps_core::eyre::Report;
use ps_core::model::Engine;

use super::app_state::UIState;
use super::platform::navigation_button;
use super::widgets::{FilterPanel, FilterState};

pub struct NavigationBar<'a> {
    engine: &'a mut Engine,
    #[allow(unused)]
    error: &'a mut Option<Report>,
    state: &'a mut UIState,
    filter_state: &'a mut FilterState,
    total_mails: usize,
}

impl<'a> NavigationBar<'a> {
    pub fn new(
        engine: &'a mut Engine,
        error: &'a mut Option<Report>,
        state: &'a mut UIState,
        filter_state: &'a mut FilterState,
        total_mails: usize,
    ) -> Self {
        NavigationBar {
            engine,
            error,
            state,
            filter_state,
            total_mails,
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

        ui.horizontal(|ui| {
            ui.set_height(40.0);

            ui.add_space(15.0);

            #[cfg(not(target_arch = "wasm32"))]
            {
                let close_text = "Close";
                if ui.add(navigation_button(close_text)).clicked() {
                    self.state.action_close = true;
                }
            }

            let filter_text = "\u{1f50D} Filters";
            let filter_response = ui.add(navigation_button(filter_text));
            let popup_id = ui.make_persistent_id("filter_panel_id");

            super::widgets::popover(ui, popup_id, &filter_response, |ui| {
                ui.add(FilterPanel::new(self.engine, self.filter_state, self.error));
            });

            ui.add(Label::new(format!(
                "{} Mails",
                self.total_mails.to_formatted_string(&Locale::en)
            )));

            // This is a hack to get right-alignment.
            // we can't size the button, we can only size text. We will size text
            // and then use ~that for these buttons
            let mut w = ui.available_width();

            let mail_text = "\u{1F4E7} Mails";
            let mail_galley = ui.painter().layout_no_wrap(
                mail_text.to_owned(),
                egui::TextStyle::Button,
                Color32::WHITE,
            );

            // FIXME: Add support for exporting the selected mails as deletion rules
            // let filter_text = "\u{1F5B9} Export";
            // let filter_galley = ui.painter().layout_no_wrap(
            //     filter_text.to_owned(),
            //     egui::TextStyle::Button,
            //     Color32::WHITE,
            // );

            w -= mail_galley.size().x + ui.spacing().button_padding.x * 4.0;
            //w -= filter_galley.size().x + ui.spacing().button_padding.x * 4.0;
            ui.add_space(w);

            //ui.add(navigation_button(filter_text));

            if ui.add(navigation_button(mail_text)).clicked() {
                self.state.show_emails = !self.state.show_emails;
            }
        })
        .response
    }
}
