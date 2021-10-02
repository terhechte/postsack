use eframe::epi::{Frame, Storage};
use eyre::{Report, Result};

use eframe::{egui, epi};

use super::state::State;
use super::widgets::Spinner;
//use crate::canvas_calc::{run, Handle, InputSender, OutputReciever, Partition, Partitions};
use crate::cluster_engine::Engine;
use crate::types::Config;

pub struct CanvasState {
    pub should_query: bool,
}

pub struct MyApp {
    config: Config,
    state: State,
    engine: Engine,
    error: Option<Report>,
}

impl MyApp {
    pub fn new(config: &Config) -> Result<Self> {
        let engine = Engine::new(&config)?;
        let state = State::new();
        Ok(Self {
            config: config.clone(),
            engine,
            state,
            error: None,
        })
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "Gmail DB"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut Frame<'_>,
        _storage: Option<&dyn Storage>,
    ) {
        self.engine.update(&self.state);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        // // If we have a selection, load the next one
        // if let Some(next) = self.next_partition.take() {
        //     self.add_partition(next);
        // }

        // // If we should recalculate
        // if self.should_recalculate {
        //     Self::recalculate(&mut self.is_querying, &self.link, &self.state.clone());
        //     self.should_recalculate = false;
        // }

        // Receive new data
        // if let Some(ref link) = self.link {

        //     // Check if the thread is still running
        //     // FIXME: Not sure how to do this, joinhandle doesn't expose anything..
        //     //if link.handle.
        // }

        self.error = self.engine.process().err();

        let Self {
            config,
            state,
            engine,
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
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("FIXME"));
                // BUG: This fails in a variety of ways. We need a better data structure
                // for this and the state/groupfield...
                /*for (index, mut stack_entry) in state.group_by_stack.iter_mut().enumerate() {
                    match (
                        partitions.len(),
                        &partitions.get(index).map(|e| e.selected.as_ref()),
                    ) {
                        (n, Some(Some(value))) if dbg!(n) > 1 => {
                            let label = egui::Label::new(format!(
                                "{}: {}",
                                &value.field.as_group_field().as_str(),
                                value.field.value()
                            ));
                            ui.add(label);
                        }
                        _ => {
                            let alternatives = State::all_group_by_fields();
                            let p = egui::ComboBox::from_id_source(&index).show_index(
                                ui,
                                &mut stack_entry,
                                alternatives.len(),
                                |i| alternatives[i].as_str().to_string(),
                            );
                            if p.changed() {
                                *should_recalculate = true;
                            }
                        }
                    }
                }
                if state.group_by_stack.len() > 1 {
                    if ui.button("Back").clicked() {
                        MyApp::remove_partition(state, partitions);
                    }
                }*/
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if engine.is_busy() {
                ui.centered_and_justified(|ui| {
                    ui.add(Spinner::new(egui::vec2(50.0, 50.0)));
                });
            } else {
                ui.add(super::widgets::Rectangles::new(engine, state));
            }
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());

        // If we're waiting for a computation to succeed,
        // we re-render again.
        // We do this because calling
        // ctx.request_repaint();
        // somehow didn't work..
        if engine.is_busy() {
            ctx.request_repaint();
        }
    }
}
