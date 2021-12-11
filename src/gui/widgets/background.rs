//! Various background utilities
use eframe::egui::{
    self,
    epaint::{RectShape, Shadow},
    vec2, Color32, Painter, Pos2, Rect, Response, Shape, Stroke, Ui, Vec2,
};

use std::ops::Rem;

use crate::gui::platform::{platform_colors, PlatformColors};

/// This will draw Ui with a background color and margins.
/// This can be used for calls that don't provide a `Frame`,
/// such as `horizontal` or `vertical`
pub fn color_background(
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
        egui::epaint::Shape::Rect(RectShape {
            corner_radius: 0.0,
            fill,
            stroke,
            rect,
        }),
    );
    ret
}

/// Draw a rectangular background with a shadow
pub fn shadow_background(
    painter: &Painter,
    paint_rect: Rect,
    fill: Color32,
    stroke: Stroke,
    corner_radius: f32,
    shadow: Shadow,
) {
    let frame_shape = Shape::Rect(RectShape {
        rect: paint_rect,
        corner_radius,
        fill,
        stroke,
    });

    let shadow = shadow.tessellate(paint_rect, corner_radius);
    let shadow = Shape::Mesh(shadow);
    let shape = Shape::Vec(vec![shadow, frame_shape]);
    painter.add(shape);
}

/// A animated backwround with some parameters.
/// Used in some `app_states`.
pub struct AnimatedBackground<'a> {
    /// The divisions
    pub divisions: usize,
    /// For each division cell, we take 8 sub divisions
    pub animate_progress: Option<(&'a [usize], usize)>,
    /// time counter
    pub timer: &'a mut f64,
    /// recursive offset counter
    pub offset_counter: &'a mut usize,
}

impl<'a> AnimatedBackground<'a> {
    pub fn draw_background(&mut self, ui: &mut egui::Ui, size: Vec2) {
        let painter = ui.painter();

        let divisions = self.divisions as f32;

        // paint stuff
        let rect_size = vec2(size.x / divisions, size.y / divisions);

        let colors = platform_colors();

        // we only animate if there's no progress
        let (offset, add) = if self.animate_progress.is_none() {
            // Define the animation speed
            let offset = *self.timer * 42.5;

            if offset > rect_size.x as f64 {
                *self.timer = 0.0;
                *self.offset_counter += 1;
            }

            // Reset the offset counter as we're going out of the size
            if (*self.offset_counter as f32 * rect_size.x) > (size.x * 1.1) {
                *self.offset_counter = 0;
            }

            // figure out the offset addition
            let add = *self.offset_counter as i8;
            (offset, add)
        } else {
            (0.0, 0)
        };

        Self::draw_rectangles(
            painter,
            offset,
            divisions,
            rect_size,
            &[
                (4 + add, 4, 3),
                (3 + add, 3, 2),
                (1 + add, 1, 5),
                (5 + add, 2, 5),
                (2 + add, 1, 6),
                (3 + add, 3, 7),
                (4 + add, 5, 1),
                (3 + add, 3, 7),
                (6 + add, 1, 3),
                (1 + add, 5, 4),
                (3 + add, 6, 5),
            ],
            self.divisions,
        );

        // Next, draw the rectangles
        if let Some((blocks, d)) = self.animate_progress {
            // the resolution of the block animation
            let divisor = self.divisions * d;
            let w = rect_size.x / d as f32;
            let h = rect_size.y / d as f32;
            let mut color_adder = self.divisions;
            for n in blocks {
                // calculate x/y from the value
                let y = n / divisor;
                let x = n % divisor;
                let y = y as f32;
                let x = x as f32;
                let pos = Pos2::new(x * w, y * h);
                let size = vec2(w, h);
                let rect = Rect::from_min_size(pos, size);
                // the fill color is based on the added block count
                color_adder += *n;
                let color_addition = if colors.is_light {
                    -((color_adder % 50) as i8)
                } else {
                    (color_adder % 50) as i8
                };
                let color = Color32::from_rgb(
                    (colors.animation_background.r() as i8 + color_addition) as u8,
                    (colors.animation_background.g() as i8 + color_addition) as u8,
                    (colors.animation_background.b() as i8 + color_addition) as u8,
                );

                painter.rect_filled(rect, 0.0, color);
                painter.rect_stroke(rect, 0.0, Stroke::new(1.0, colors.line3));
            }
        }

        let diff = ui.input().unstable_dt as f64;
        *self.timer += diff;

        ui.ctx().request_repaint();
    }

    fn draw_rectangles(
        painter: &Painter,
        offset: f64,
        division: f32,
        size: Vec2,
        recurse: &[(i8, i8, i8)],
        total: usize,
    ) {
        let colors = platform_colors();

        for y in 0..=(division + 2.0) as i8 {
            for x in 0..=(division + 2.0) as i8 {
                let fx = ((x - 1) as f32 * size.x) + (offset as f32);
                let fy = (y - 1) as f32 * size.y;
                let pos = Pos2::new(fx, fy);
                let rect = Rect::from_min_size(pos, size);
                painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(70)));
                for (rx, ry, rd) in recurse {
                    // on the x axis take the offset into account
                    let rx = (*rx).rem((total as i8) + 1);
                    if rx == x && ry == &y {
                        Self::draw_segmentation(painter, rect, *rd, colors);
                    }
                }
            }
        }
    }

    fn draw_segmentation(painter: &Painter, into: Rect, divisions: i8, colors: &PlatformColors) {
        let mut rect = into;
        for d in 0..=divisions {
            // division back and forth in direction
            let next = if d % 2 == 0 {
                Rect::from_min_size(
                    Pos2 {
                        x: rect.center().x,
                        y: rect.top(),
                    },
                    Vec2 {
                        x: rect.width() / 2.0,
                        y: rect.height(),
                    },
                )
            } else {
                Rect::from_min_size(
                    Pos2 {
                        x: rect.left(),
                        y: rect.center().y,
                    },
                    Vec2 {
                        x: rect.width(),
                        y: rect.height() / 2.0,
                    },
                )
            };
            painter.rect_stroke(next, 0.0, Stroke::new(1.0, colors.line4));
            rect = next;
        }
    }
}
