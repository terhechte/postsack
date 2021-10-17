/// This will draw Ui with a background color and margins.
/// This can be used for calls that don't provide a `Frame`,
/// such as `horizontal` or `vertical`
use eframe::egui::{self, Color32, Rect, Response, Stroke, Ui};

pub fn background_color(
    ui: &mut Ui,
    padding: f32,
    stroke: Stroke,
    fill: Color32,
    show: impl FnOnce(&mut Ui) -> Response,
) -> Response {
    let outer_rect_bounds = ui.available_rect_before_wrap();
    let where_to_put_background = ui.painter().add(egui::Shape::Noop);
    let margin = egui::Vec2::splat(padding);
    let inner_rect = outer_rect_bounds.shrink2(margin);

    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = show(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, _) = ui.allocate_at_least(outer_rect.size(), egui::Sense::hover());

    ui.painter().set(
        where_to_put_background,
        egui::epaint::Shape::Rect {
            corner_radius: 0.0,
            fill,
            stroke,
            rect,
        },
    );
    ret
}
