use std::collections::hash_map::DefaultHasher;

use crate::model::{segmentations, Engine, Segment};
use eframe::egui::{self, epaint::Galley, Pos2, Rgba, Stroke, TextStyle, Widget};
use eyre::Report;
use num_format::{Locale, ToFormattedString};

fn segment_to_color(segment: &Segment) -> Rgba {
    let mut hasher = DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    let value = segment.field.value().to_string();
    value.hash(&mut hasher);
    let value = hasher.finish();
    let [r1, r2, g1, g2, b1, b2, _, _] = value.to_be_bytes();

    Rgba::from_rgb(
        (r1 as f32 + r2 as f32) / (u8::MAX as f32 * 2.0),
        (g1 as f32 + g2 as f32) / (u8::MAX as f32 * 2.0),
        (b1 as f32 + b2 as f32) / (u8::MAX as f32 * 2.0),
    )
}

pub struct Rectangles<'a> {
    engine: &'a mut Engine,
    error: &'a mut Option<Report>,
}

impl<'a> Rectangles<'a> {
    pub fn new(engine: &'a mut Engine, error: &'a mut Option<Report>) -> Self {
        Rectangles { engine, error }
    }
}

impl<'a> Widget for Rectangles<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = ui.available_size();
        let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::hover());

        let items = match segmentations::layouted_segments(self.engine, rect) {
            Some(n) => n.to_owned(),
            None => return response,
        };

        for item in items {
            let item_response = ui.put(item.layout_rect(), rectangle(&item));
            if item_response.clicked() {
                *self.error = self.engine.push(item.clone()).err();
                response.mark_changed();
            }
        }

        response
    }
}

fn rectangle_ui(ui: &mut egui::Ui, segment: &Segment) -> egui::Response {
    let size = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    let visuals = ui.style().interact_selectable(&response, true);

    let stroke = if ui.ui_contains_pointer() {
        Stroke::new(4.0, visuals.fg_stroke.color)
    } else {
        Stroke::default()
    };

    let color = segment_to_color(segment);

    let painter = ui.painter();

    painter.rect(rect, 0.0, color, stroke);
    let mut center = rect.center();

    let align_bottom = |galley: &std::sync::Arc<Galley>, center: &mut Pos2, spacing: f32| {
        #[allow(clippy::clone_on_copy)]
        let mut position = center.clone();
        position.x -= galley.size.x / 2.0;
        position.y -= galley.size.y / 2.0;
        center.y += galley.size.y + spacing;
        if galley.size.x < rect.width() && galley.size.y < rect.height() {
            Some(position)
        } else {
            None
        }
    };

    // Write the label and the amount
    {
        let text = format!("{}", segment.field.value());
        let galley = painter.layout_no_wrap(TextStyle::Body, text);
        let previous_center = center;
        if let Some(center) = align_bottom(&galley, &mut center, 2.0) {
            painter.galley(center, galley, Rgba::BLACK.into());
        } else {
            // If the name doesn't fit, reverse the changes to center the count
            center = previous_center;
        }
    }
    {
        let text = segment.count.to_formatted_string(&Locale::en);
        let galley = painter.layout_no_wrap(TextStyle::Small, text);
        if let Some(center) = align_bottom(&galley, &mut center, 5.0) {
            painter.galley(center, galley, Rgba::BLACK.into());
        }
    }
    let label = format!("{}\n{}", &segment.field.value(), &segment.count);

    response.on_hover_text(&label)
}

fn rectangle(segment: &Segment) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| rectangle_ui(ui, segment)
}
