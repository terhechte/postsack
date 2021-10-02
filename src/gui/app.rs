use eframe::epi::{Frame, Storage};
use eyre::{Report, Result};

use eframe::{egui, epi};

use super::state::State;
use super::widgets::Spinner;
use crate::canvas_calc::{run, Handle, InputSender, OutputReciever, Partition, Partitions};
use crate::types::Config;

struct Link {
    input_sender: InputSender,
    output_receiver: OutputReciever,
    handle: Handle,
}

pub struct CanvasState {
    pub should_query: bool,
}

pub struct MyApp {
    config: Config,
    link: Option<Link>,
    state: State,
    partitions: Vec<Partitions>,
    next_partition: Option<Partition>,
    error: Option<Report>,
    is_querying: bool,
}

impl MyApp {
    pub fn new(config: &Config) -> Self {
        let state = State::new();
        Self {
            config: config.clone(),
            link: None,
            state,
            //canvas_state: CanvasState {
            partitions: Vec::new(),
            next_partition: None,
            //should_query: false,
            //},
            error: None,
            is_querying: false,
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
        let (input_sender, output_receiver, handle) = match run(&self.config) {
            Ok(n) => n,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };

        input_sender.send(self.state.clone());

        self.link = Some(Link {
            input_sender,
            output_receiver,
            handle,
        });
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        // If we have a selection, load the next one
        if let Some(next) = self.next_partition.take() {
            self.add_partition(next);
        }

        // Receive new data
        if let Some(ref link) = self.link {
            match link.output_receiver.try_recv() {
                Ok(Ok(p)) => {
                    self.partitions.push(p);
                    self.is_querying = false;
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
            is_querying: is_rendering,
            next_partition,
            error,
            ..
        } = self;

        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            ui.heading("GMail DB");
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
            if state.group_by_stack.len() > 1 {
                if ui.button("Back").clicked() {
                    MyApp::remove_partition(state, partitions);
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match (partitions.last_mut(), *is_rendering) {
                (_, true) | (None, false) => {
                    ui.centered_and_justified(|ui| {
                        ui.add(Spinner::new(egui::vec2(50.0, 50.0)));
                    });
                }
                (Some(p), _) => {
                    //ui.add(super::widgets::rectangles(p));
                    ui.add(super::widgets::Rectangles {
                        partitions: p,
                        select_next: next_partition,
                    });
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

impl MyApp {
    fn add_partition(&mut self, partition: Partition) -> Option<()> {
        // Assign the partition
        let current = self.partitions.last_mut()?;
        current.selected = Some(partition);

        // Create the new search stack
        self.state.search_stack = self
            .partitions
            .iter()
            .filter_map(|e| e.selected.as_ref())
            .map(|p| p.field.clone())
            .collect();

        // Add the next group by
        let index = self.state.group_by_stack.len();
        let next = super::state::default_group_by_stack(index);
        self.state.group_by_stack.push(next);

        // Submit it
        if let Some(ref link) = self.link {
            link.input_sender.send(self.state.clone());
        }

        // Block UI & Wait for updates
        self.is_querying = true;
        Some(())
    }

    pub fn remove_partition(state: &mut State, partitions: &mut Vec<Partitions>) {
        // FIXME: Checks
        state.group_by_stack.remove(state.group_by_stack.len() - 1);
        partitions.remove(partitions.len() - 1);
        state.search_stack.remove(state.search_stack.len() - 1);
    }
}
