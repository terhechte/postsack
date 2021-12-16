//! A popover is a popup that only closes when clicking outside.
//! It is lifted from:
//! https://github.com/emilk/egui/blob/a1bf5aff47a7f6f3d698e6ccfb7b62b65ef2de5b/egui/src/widgets/color_picker.rs
//! Line 355.
//!
use eframe::egui::{self, Id, Response, Ui};

pub fn popover(
    ui: &Ui,
    popup_id: Id,
    widget_response: &Response,
    add_contents: impl FnOnce(&mut Ui),
) {
    if widget_response.clicked() {
        ui.memory().toggle_popup(popup_id);
    }

    if ui.memory().is_popup_open(popup_id) {
        let area_response = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .default_pos(widget_response.rect.left_bottom())
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, add_contents);
            })
            .response;

        if !widget_response.clicked()
            && (ui.input().key_pressed(egui::Key::Escape) || area_response.clicked_elsewhere())
        {
            ui.memory().close_popup();
        }

        //Some(area_response)
    }
}
