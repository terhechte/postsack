//! The startup form to configure what and how to import
use eframe::egui::epaint::Shadow;
use eframe::egui::{self, vec2, Color32, Pos2, Rect, Response, Stroke, TextStyle, Vec2, Widget};
use rand::seq::SliceRandom;

use super::super::platform::platform_colors;
use super::super::widgets::background::{shadow_background, AnimatedBackground};
use crate::types::Config;
use crate::types::FormatType;

pub struct Import {
    config: Config,
    /// The animation divisions
    animation_divisions: usize,
    /// time counter
    timer: f64,
    /// recursive offset counter
    offset_counter: usize,
    /// We use this to have the initial background resize
    /// animation
    intro_timer: f64,
    /// This defines the amount of progress blocks we intend
    /// to animate
    progress_blocks: Vec<usize>,
    /// The progress divisions
    progress_divisions: usize,
}

impl Import {
    pub fn new(config: Config) -> Self {
        // Build a random distribution of elements
        // to animate the import process
        let mut rng = rand::thread_rng();
        let animation_divisions = 6;
        let progress_divisions = 4;

        // the amount of progress blocks
        let progress_block_count =
            (animation_divisions * progress_divisions) * (animation_divisions * progress_divisions);
        let mut progress_blocks: Vec<usize> = (0..progress_block_count).collect();
        dbg!(progress_block_count);
        progress_blocks.shuffle(&mut rng);

        Self {
            animation_divisions,
            config,
            timer: 0.0,
            offset_counter: 0,
            intro_timer: 0.0,
            progress_blocks,
            progress_divisions,
        }
    }
}

impl Widget for &mut Import {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        self.intro_timer += ui.input().unstable_dt as f64;
        let growth = self.intro_timer.clamp(0.0, 1.0);

        let available = ui.available_size();

        // We take the progress as a value fromt the blocks
        // FIXME: temporary using intro timer
        let p = ((self.intro_timer * 5.0) / 100.0);
        let n = (self.progress_blocks.len() as f64 * p) as usize;
        //println!("{} / {}", n, self.progress_blocks.len());
        let slice = &self.progress_blocks[0..=n];

        AnimatedBackground {
            divisions: self.animation_divisions,
            animate_progress: Some((slice, self.progress_divisions)),
            timer: &mut self.timer,
            offset_counter: &mut self.offset_counter,
        }
        .draw_background(ui, available);

        let desired_height = 370.0 - (270.0 * growth) as f32;
        let desired_size = egui::vec2(330.0, desired_height);

        let paint_rect = Rect::from_min_size(
            Pos2 {
                x: available.x / 2.0 - desired_size.x / 2.0,
                y: available.y / 2.0 - desired_size.y / 2.0,
            },
            desired_size,
        );

        // calculate in margin
        let center = paint_rect.shrink(15.0);

        let colors = platform_colors();

        // Draw a backround with a shadow
        shadow_background(
            ui.painter(),
            paint_rect,
            colors.window_background_dark,
            Stroke::new(1.0, Color32::from_gray(90)),
            12.0,
            Shadow::big_dark(),
        );

        ui.allocate_ui_at_rect(center, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.heading("Import in Progress");
                    let bar = egui::widgets::ProgressBar::new(0.5).animate(true);
                    ui.add(bar);
                    ui.small("133 / 1000");
                });
            })
        })
        .response
    }
}
