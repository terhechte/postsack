//! Some functions to generate beautiful colors based on a hash.
//! loosely based on https://github.com/BrandtM/colourado/blob/master/src/lib.rs

use eframe::egui::Color32;

pub fn color(input_value: u64, total: usize, number: usize) -> Color32 {
    let [r1, r2, g1, g2, b1, b2, _, _] = input_value.to_be_bytes();
    let mut hue = (r1 as f32 + r2 as f32) / (u8::MAX as f32 * 2.0) * 360.0;
    let saturation = ((g1 as f32 + g2 as f32) / (u8::MAX as f32 * 2.0) * 0.3) + 0.1;
    let value = ((b1 as f32 + b2 as f32) / (u8::MAX as f32 * 2.0) * 0.3) + 0.7;

    let mut base_divergence = 80.0;
    base_divergence -= (total as f32) / 2.6;

    let number = number as f32;

    let f = (number * 25.0).cos().abs();
    let mut div = base_divergence;

    if div < 15.0 {
        div = 15.0;
    }

    hue = (hue + div + f).abs() % 360.0;
    // Enabling this makes the colors more pasteley.
    // maybe a preference?
    //saturation = ((number * 0.35).cos() / 5.0).abs();
    //value = 0.5 + (number.cos() / 2.0).abs();

    hsv_to_rgb(hue, saturation, value)
}

trait InRange {
    fn in_range(&self, begin: Self, end: Self) -> bool;
}

impl InRange for f32 {
    fn in_range(&self, begin: f32, end: f32) -> bool {
        *self >= begin && *self < end
    }
}

/// Convert HSV to RGB. Plain and simple
fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Color32 {
    let chroma = value * saturation;
    let hue2 = hue / 60.0;
    let tmp = chroma * (1.0 - ((hue2 % 2.0) - 1.0).abs());
    let color2: (f32, f32, f32);

    match hue2 {
        h if h.in_range(0.0, 1.0) => color2 = (chroma, tmp, 0.0),
        h if h.in_range(1.0, 2.0) => color2 = (tmp, chroma, 0.0),
        h if h.in_range(2.0, 3.0) => color2 = (0.0, chroma, tmp),
        h if h.in_range(3.0, 4.0) => color2 = (0.0, tmp, chroma),
        h if h.in_range(4.0, 5.0) => color2 = (tmp, 0.0, chroma),
        h if h.in_range(5.0, 6.0) => color2 = (chroma, 0.0, tmp),
        _ => color2 = (0.0, 0.0, 0.0),
    }

    let m = value - chroma;
    let red = color2.0 + m;
    let green = color2.1 + m;
    let blue = color2.2 + m;

    Color32::from_rgb(
        (red * 255.0) as u8,
        (green * 255.0) as u8,
        (blue * 255.0) as u8,
    )
}
