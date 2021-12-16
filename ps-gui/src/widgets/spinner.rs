use eframe::egui::{
    self, epaint::PathShape, lerp, vec2, Color32, Pos2, Response, Shape, Stroke, Vec2, Widget,
};

/// A simple spinner
pub struct Spinner(Vec2);

impl Spinner {
    pub fn new(size: Vec2) -> Self {
        Self(size)
    }
}

impl Widget for Spinner {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let Spinner(size) = self;

        let (outer_rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
        let visuals = ui.style().visuals.clone();

        let corner_radius = outer_rect.height() / 2.0;

        let n_points = 20;
        let start_angle = ui.input().time as f64 * 360f64.to_radians();
        let end_angle = start_angle + 240f64.to_radians() * ui.input().time.sin();
        let circle_radius = corner_radius - 2.0;
        let points: Vec<Pos2> = (0..n_points)
            .map(|i| {
                let angle = lerp(start_angle..=end_angle, i as f64 / n_points as f64);
                let (sin, cos) = angle.sin_cos();
                outer_rect.right_center()
                    + circle_radius * vec2(cos as f32, sin as f32)
                    + vec2(-corner_radius, 0.0)
            })
            .collect();
        let shape = Shape::Path(PathShape {
            points,
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(8.0, visuals.strong_text_color()),
        });
        ui.painter().add(shape);

        ui.ctx().request_repaint();

        response
    }
}
