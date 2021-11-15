use std::collections::hash_map::DefaultHasher;

use crate::model::{segmentations, Engine, Segment};
use eframe::egui::{self, epaint::Galley, Color32, Pos2, Rect, Rgba, Stroke, TextStyle, Widget};
use eyre::Report;
use num_format::{Locale, ToFormattedString};

use super::super::platform::platform_colors;

fn segment_to_color(segment: &Segment, total: usize, position: usize) -> Color32 {
    let mut hasher = DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    let value = segment.field.value().to_string();
    value.hash(&mut hasher);
    let value = hasher.finish();
    super::color_utils::color(value, total, position)
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

        let active = crate::model::segmentations::can_aggregate_more(self.engine);

        let colors = platform_colors();

        let total = items.len();
        let mut hovered: Option<String> = None;
        for (index, item) in items.iter().enumerate() {
            let item_response = ui.put(
                item.layout_rect(),
                rectangle(&item, active, colors.content_background, index, total),
            );
            if item_response.clicked() && active {
                *self.error = self.engine.push(item.clone()).err();
                response.mark_changed();
            }
            if item_response.hovered() {
                hovered = Some(format!("#{}: {}", item.count, item.field.to_string()));
            }
        }

        if let Some(h) = hovered {
            // Calculate the size
            let text = format!("{}", h);
            let galley = ui.painter().layout_no_wrap(TextStyle::Body, text.clone());

            // keep spacing in mind
            let size: Pos2 = (
                galley.size.x + ui.spacing().button_padding.x * 2.0,
                galley.size.y + ui.spacing().button_padding.y * 2.0,
            )
                .into();

            // we build a disabled for easy rounded corners
            let label_button = egui::widgets::Button::new(text)
                .enabled(false)
                .text_color(Color32::WHITE);

            // we want to be a wee bit in the rectangle system
            let offset = -2.0;
            ui.put(
                Rect::from_min_size(
                    (
                        rect.left() - offset,
                        (rect.bottom() + offset) - (size.y + 10.0),
                    )
                        .into(),
                    (size.x + 10.0, size.y + 10.0).into(),
                ),
                label_button,
            );
        }

        response
    }
}

fn rectangle_ui(
    ui: &mut egui::Ui,
    segment: &Segment,
    active: bool,
    stroke_color: Color32,
    position: usize,
    total: usize,
) -> egui::Response {
    let size = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    //let visuals = ui.style().interact_selectable(&response, true);

    let stroke = Stroke::new(1.0, stroke_color);

    let color = segment_to_color(segment, total, position);
    let color = if ui.ui_contains_pointer() && active {
        Color32::from_rgb(
            color.r().saturating_add(25),
            color.g().saturating_add(25),
            color.b().saturating_add(25),
        )
    } else {
        color
    };

    let painter = ui.painter();

    painter.rect(rect, 2.0, color, stroke);
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
        // Take the max width - some spacing to fit into the rectangle
        let width = rect.width() - ui.spacing().button_padding.x * 2.0;
        let text = segment.field.to_string();
        let galley = painter.layout_multiline(TextStyle::Body, text, width);
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
    response
}

fn rectangle(
    segment: &Segment,
    active: bool,
    stroke_color: Color32,
    position: usize,
    total: usize,
) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| rectangle_ui(ui, segment, active, stroke_color, position, total)
}
