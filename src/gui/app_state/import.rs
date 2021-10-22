//! The startup form to configure what and how to import
use std::thread::JoinHandle;

use eframe::egui::epaint::Shadow;
use eframe::egui::{self, Color32, Pos2, Rect, Response, Stroke};
use eyre::Result;
use rand::seq::SliceRandom;

use super::super::platform::platform_colors;
use super::super::widgets::background::{shadow_background, AnimatedBackground};
use super::{StateUIAction, StateUIVariant};
use crate::types::Config;
use crate::{
    importer::{self, Adapter, State},
    types::FormatType,
};

pub struct ImporterUI {
    /// The config for this configuration
    config: Config,
    /// The adapter handling the import
    adapter: Adapter,
    /// The handle to the adapter thread
    /// As handle.join takes `self` it has to be optional
    handle: Option<JoinHandle<Result<()>>>,
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
    /// we're done importing
    pub done_importing: bool,
    /// Any errors during importing
    pub importer_error: Option<eyre::Report>,
}

impl ImporterUI {
    pub fn new(config: Config) -> Result<Self> {
        let cloned_config = config.clone();
        // Build a random distribution of elements
        // to animate the import process
        let mut rng = rand::thread_rng();
        let animation_divisions = 6;
        let progress_divisions = 4;

        // the amount of progress blocks
        let progress_block_count =
            (animation_divisions * progress_divisions) * (animation_divisions * progress_divisions);
        let mut progress_blocks: Vec<usize> = (0..progress_block_count).collect();
        progress_blocks.shuffle(&mut rng);

        // The adapter that controls the syncing
        let adapter = Adapter::new();

        // Could not figure out how to build this properly
        // with dynamic dispatch. (to abstract away the match)
        // Will try again when I'm online.
        let handle = match config.format {
            FormatType::AppleMail => {
                let importer = importer::applemail_importer(config);
                adapter.process(importer)?
            }
            FormatType::GmailVault => {
                let importer = importer::gmail_importer(config);
                adapter.process(importer)?
            }
            FormatType::Mbox => {
                let importer = importer::mbox_importer(config);
                adapter.process(importer)?
            }
        };

        Ok(Self {
            config: cloned_config,
            adapter,
            handle: Some(handle),
            animation_divisions,
            timer: 0.0,
            offset_counter: 0,
            intro_timer: 0.0,
            progress_blocks,
            progress_divisions,
            done_importing: false,
            importer_error: None,
        })
    }
}
impl StateUIVariant for ImporterUI {
    fn update_panel(&mut self, ctx: &egui::CtxRef) -> StateUIAction {
        egui::CentralPanel::default()
            .frame(egui::containers::Frame::none())
            .show(ctx, |ui| {
                ui.add(|ui: &mut egui::Ui| self.ui(ui));
            });
        // If we generated an action above, return it
        match (self.importer_error.take(), self.done_importing) {
            (Some(report), _) => StateUIAction::Error {
                report,
                config: self.config.clone(),
            },
            (_, true) => StateUIAction::ImportDone {
                config: self.config.clone(),
            },
            (_, false) => StateUIAction::Nothing,
        }
    }
}

impl ImporterUI {
    fn ui(&mut self, ui: &mut egui::Ui) -> Response {
        // The speed with which we initially scale down.
        self.intro_timer += (ui.input().unstable_dt as f64) * 2.0;
        let growth = self.intro_timer.clamp(0.0, 1.0);

        let available = ui.available_size();

        let (label, progress, writing, done) = match self.handle_adapter() {
            Ok(n) => n,
            Err(e) => {
                // Generate a response signifying we're done - as there was an error
                let response = (format!("Error {}", &e), 1.0, false, true);
                self.importer_error = Some(e);
                response
            }
        };

        if let Ok(Some(error)) = self.adapter.error() {
            self.importer_error = Some(error);
        }

        if done {
            // if we're done, the join handle should not lock
            if let Some(handle) = self.handle.take() {
                self.importer_error = handle.join().ok().map(|e| e.err()).flatten();
            }
            self.done_importing = true;
        }

        let n = (self.progress_blocks.len() as f32 * progress) as usize;
        let n = n.min(self.progress_blocks.len());
        let slice = &self.progress_blocks[0..n];

        AnimatedBackground {
            divisions: self.animation_divisions,
            animate_progress: Some((slice, self.progress_divisions)),
            timer: &mut self.timer,
            offset_counter: &mut self.offset_counter,
        }
        .draw_background(ui, available);

        let desired_height = 370.0 - (220.0 * growth) as f32;
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
                    ui.add_space(10.0);
                    if writing {
                        let bar = egui::widgets::ProgressBar::new(1.0).animate(false);
                        ui.add(bar);
                        let bar = egui::widgets::ProgressBar::new(progress).animate(true);
                        ui.add(bar);
                    } else {
                        let bar = egui::widgets::ProgressBar::new(progress).animate(true);
                        ui.add(bar);
                        ui.add_space(20.0);
                    }
                    ui.small(label);
                });
            })
        })
        .response
    }
}

impl ImporterUI {
    /// Returns the current label, the progress (0-1), writing? (true), and done? (true)
    fn handle_adapter(&mut self) -> Result<(String, f32, bool, bool)> {
        let (mut label, progress, writing) = {
            let write = self.adapter.write_count()?;
            if write.count > 0 {
                (
                    format!("\rParsing emails {}/{}...", write.count, write.total),
                    (write.count as f32 / write.total as f32),
                    true,
                )
            } else {
                let read = self.adapter.read_count()?;
                (
                    format!("Reading emails {}/{}...", read.count, read.total),
                    (read.count as f32 / read.total as f32),
                    false,
                )
            }
        };

        let State { done, finishing } = self.adapter.finished()?;
        if finishing {
            label = format!("Finishing Up");
        }
        Ok((label, progress, writing, done))
    }
}
