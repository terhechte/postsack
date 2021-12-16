/// Sort of mirror `egui::rect` for simplicity
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(min: (f64, f64), max: (f64, f64)) -> Rect {
        Rect {
            left: min.0,
            top: min.1,
            width: max.0 - min.0,
            height: max.1 - min.1,
        }
    }
}
