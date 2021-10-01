use eframe::epi::{Frame, Storage};
use eyre::{Report, Result};

use eframe::{egui, epi};

use super::state::State;
use super::widgets::Spinner;
use crate::canvas_calc::{run, Handle, InputSender, OutputReciever, Partitions};
use crate::types::Config;

struct Link {
    input_sender: InputSender,
    output_receiver: OutputReciever,
    handle: Handle,
}

pub struct MyApp {
    config: Config,
    link: Option<Result<Link>>,
    state: State,
    partitions: Vec<Partitions>,
    error: Option<Report>,
    is_rendering: bool,
}

impl MyApp {
    pub fn new(config: &Config) -> Self {
        let state = State {
            year_filter: None,
            domain_filter: None,
        };
        Self {
            config: config.clone(),
            link: None,
            state,
            partitions: Vec::new(),
            error: None,
            is_rendering: false,
        }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut Frame<'_>,
        _storage: Option<&dyn Storage>,
    ) {
        let link = run(&self.config).map(|(input_sender, output_receiver, handle)| Link {
            input_sender,
            output_receiver,
            handle,
        });

        if let Ok(l) = &link {
            l.input_sender.send(self.state.clone());
        }

        self.link = Some(link);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        // If we have a selection, load the next one
        if let Some(partition) = self.partitions.last() {
            if let Some(sel) = &partition.selected {
                if let Some(Ok(ref link)) = self.link {
                    link.input_sender.send(self.state.clone());
                }
                self.is_rendering = true;
            }
        }

        // Receive new data
        if let Some(Ok(ref link)) = self.link {
            match link.output_receiver.try_recv() {
                Ok(Ok(p)) => {
                    self.partitions.push(p);
                    self.is_rendering = false;
                }
                Err(_) => (),
                Ok(Err(e)) => self.error = Some(e),
            }

            // Check if the thread is still running
            // FIXME: Not sure how to do this, joinhandle doesn't expose anything..
            //if link.handle.
        }

        let Self {
            config,
            link,
            state,
            partitions,
            is_rendering,
            error,
            ..
        } = self;

        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                ui.label("Search");
                //ui.text_edit_singleline(state.domain_filter);
            });
            //ui.add(egui::Slider::new(age, 0..=120).text("age"));
            //if ui.button("Click each year").clicked() {
            //    if let Some(Ok(link)) = link {
            //        link.input_sender.send(state.clone());
            //        *is_rendering = true;
            //    }
            //}
        });

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.label("GmailDB");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match (partitions.last_mut(), *is_rendering) {
                (_, true) | (None, false) => {
                    ui.centered_and_justified(|ui| {
                        ui.add(Spinner::new(egui::vec2(50.0, 50.0)));
                    });
                }
                (Some(p), _) => {
                    ui.add(super::widgets::rectangles(p));
                }
            }
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());

        // If we're waiting for a computation to succeed,
        // we re-render again.
        // We do this because calling
        // ctx.request_repaint();
        // somehow didn't work..
        if *is_rendering == true {
            ctx.request_repaint();
        }
    }
}
