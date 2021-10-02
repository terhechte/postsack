use std::collections::hash_map::DefaultHasher;

use crate::cluster_engine::{Engine, IntoRequest, Partition};
use eframe::egui::{self, epaint::Galley, Pos2, Rgba, Stroke, TextStyle, Widget};
use num_format::{Locale, ToFormattedString};

fn partition_to_color(partition: &Partition) -> Rgba {
    let mut hasher = DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    partition.field.hash(&mut hasher);
    let value = hasher.finish();
    let [r1, r2, g1, g2, b1, b2, _, _] = value.to_be_bytes();

    Rgba::from_rgb(
        (r1 as f32 + r2 as f32) / (u8::MAX as f32 * 2.0),
        (g1 as f32 + g2 as f32) / (u8::MAX as f32 * 2.0),
        (b1 as f32 + b2 as f32) / (u8::MAX as f32 * 2.0),
    )
}

pub struct Rectangles<'a, S: IntoRequest> {
    engine: &'a mut Engine,
    state: &'a S,
}

impl<'a, S: IntoRequest> Rectangles<'a, S> {
    pub fn new(engine: &'a mut Engine, state: &'a S) -> Self {
        Rectangles { engine, state }
    }
}

#[allow(unused_must_use)]
impl<'a, S: IntoRequest> Widget for Rectangles<'a, S> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = ui.available_size();
        let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::hover());

        let items = match self.engine.items_with_size(rect) {
            Some(n) => n.to_owned(),
            None => return response,
        };

        for item in items {
            let item_response = ui.put(item.layout_rect(), rectangle(&item));
            if item_response.clicked() {
                self.engine.select_partition(item.clone(), self.state);
                response.mark_changed();
            }
        }

        response
    }
}

fn rectangle_ui(ui: &mut egui::Ui, partition: &Partition) -> egui::Response {
    let size = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    let visuals = ui.style().interact_selectable(&response, true);

    let stroke = if ui.ui_contains_pointer() {
        Stroke::new(4.0, visuals.fg_stroke.color)
    } else {
        Stroke::default()
    };

    let color = partition_to_color(&partition);

    let painter = ui.painter();

    painter.rect(rect, 0.0, color, stroke);
    let mut center = rect.center();

    let align_bottom = |galley: &std::sync::Arc<Galley>, center: &mut Pos2, spacing: f32| {
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
        let text = format!("{}", partition.field.value());
        let galley = painter.layout_no_wrap(TextStyle::Body, text);
        if let Some(center) = align_bottom(&galley, &mut center, 2.0) {
            painter.galley(center, galley, Rgba::BLACK.into());
        }
    }
    {
        let text = partition.count.to_formatted_string(&Locale::en);
        let galley = painter.layout_no_wrap(TextStyle::Small, text);
        if let Some(center) = align_bottom(&galley, &mut center, 5.0) {
            painter.galley(center, galley, Rgba::BLACK.into());
        }
    }
    let label = format!("{}\n{}", &partition.field.value(), &partition.count);

    response.on_hover_text(&label)
}

fn rectangle(partition: &Partition) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| rectangle_ui(ui, partition)
}
