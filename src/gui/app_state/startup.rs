use eframe::egui::epaint::Shadow;
use eframe::egui::{
    self, vec2, Color32, Painter, Pos2, Rect, Response, Shape, Stroke, TextStyle, Vec2, Widget,
};
use rfd;

use std::ops::Rem;
use std::path::PathBuf;

use super::super::platform::platform_colors;
use crate::types::FormatType;

#[derive(Default)]
pub struct Startup {
    /// Which importer format are we using
    format: FormatType,
    /// Where are the emails located
    email_folder: Option<PathBuf>,
    /// Should we keep them in memory,
    /// or save them to disk, to this location
    database_path: Option<PathBuf>,
    /// Should we save to disk as a flag
    save_to_disk: bool,
    /// The email address of the user
    email_address: Option<String>,
    /// time counter
    timer: f64,
    /// recursive offset counter
    offset_counter: usize,
}

impl Widget for &mut Startup {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let available = ui.available_size();

        self.draw_background(ui, available);

        // I did not find an easy solution to center a frame in
        // on the vertical and horizontal position.
        // I tried `ui.centered_and_justified`,
        // `ui.allocate_exact_size`
        // `ui.allocate_with_layout`
        // and variations. This, at least, worked.
        let desired_size = egui::vec2(330.0, 370.0);

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

        let corner_radius = 12.0;

        let frame_shape = Shape::Rect {
            rect: paint_rect,
            corner_radius,
            fill: colors.window_background_dark,
            stroke: Stroke::new(1.0, Color32::from_gray(90)),
        };

        let shadow = Shadow::big_dark().tessellate(paint_rect, 8.0);
        let shadow = Shape::Mesh(shadow);
        let shape = Shape::Vec(vec![shadow, frame_shape]);
        ui.painter().add(shape);

        // placeholder text
        let mut txt = "john@example.org".to_string();

        let response = ui.allocate_ui_at_rect(center, |ui| {
            // We use a grid as that gives us more spacing opportunities
            egui::Grid::new("filter_grid")
                .spacing(vec2(15.0, 12.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::widgets::Label::new("Choose Import Format:")
                            .text_color(Color32::WHITE)
                            .text_style(TextStyle::Body),
                    );
                    ui.end_row();

                    self.format_selection(ui, center.width() * 0.7);
                    ui.end_row();

                    ui.add(
                        egui::widgets::Label::new("Email Folder:")
                            .text_color(Color32::WHITE)
                            .text_style(TextStyle::Body),
                    );
                    ui.end_row();

                    ui.horizontal(|ui| {
                        if ui.button("Browse...").clicked() {
                            self.open_email_folder_dialog()
                        }
                        if self.format == FormatType::AppleMail {
                            if ui.button("or Mail.app default folder").clicked() {
                                self.email_folder = self.format.default_path().map(|e| e.to_path_buf())
                            }
                        }
                    });
                    ui.end_row();
                    if let Some(n) = self.email_folder.as_ref() {
                        ui.label(format!("{}", n.display()))
                            .on_hover_text(format!("{}", self.email_folder.as_ref().unwrap().display()));
                    }
                    ui.end_row();

                    ui.add(
                        egui::widgets::Label::new("Your Email Address:").text_color(Color32::WHITE),
                    );
                    ui.end_row();

                    let response = ui.text_edit_singleline(&mut txt);
                    if response.changed() {
                        self.email_address = Some(txt);
                    }

                    ui.small_button("?")
                        .on_hover_text("Multiple addresses can be\nseparated by comma (,)");
                    ui.end_row();

                    ui.add(
                        egui::widgets::Label::new("Used to filter send mails")
                            .text_style(TextStyle::Small),
                    );
                    ui.end_row();

                    ui.checkbox(&mut self.save_to_disk, "Save Imported Output Database?");
                    ui.small_button("?").on_hover_text(
                        "Save the database generated\nduring import. It can be opened\nwith the \"Open Database\" \nbutton below",
                    );
                    ui.end_row();

                    if self.save_to_disk {
                        ui.horizontal(|ui| {
                            if ui.button("Output Location").clicked() {
                                self.open_database_dialog()
                            }
                            if let Some(Some(Some(name))) = self.database_path.as_ref().map(|e| e.file_name().map(|e| e.to_str().map(|e| e.to_string()))) {
                                ui.add(egui::widgets::Label::new(name));
                            }
                        });
                    }
                    ui.end_row();

                    // FIXME: Only true if all data is set
                    if true {
                        let button_size1: Vec2 = ((center.width() / 2.0) - 25.0, 25.0).into();
                        let button_size2: Vec2 = ((center.width() / 2.0) - 25.0, 25.0).into();
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                button_size1,
                                egui::Button::new("Start").text_color(Color32::WHITE),
                            );
                            ui.add_sized(button_size2, egui::Button::new("Or Open Database"));
                        });
                    }
                    ui.end_row();
                });
        });

        response.response
    }
}

impl Startup {
    fn format_selection(&mut self, ui: &mut egui::Ui, width: f32) {
        let mut selected = self.format;
        let response = egui::ComboBox::from_id_source("mailbox_type_comboox")
            .width(width)
            .selected_text(format!("{:?}", selected.name()))
            .show_ui(ui, |ui| {
                for format in FormatType::all_cases() {
                    ui.selectable_value(&mut selected, format, format.name());
                }
            });
        if response.response.changed() {
            self.format = selected;
        }
    }

    fn open_email_folder_dialog(&mut self) {
        let default_path = self
            .format
            .default_path()
            .unwrap_or(std::path::Path::new("~/"));

        let folder = rfd::FileDialog::new()
            .set_directory(default_path)
            .pick_folder();

        let path = match folder {
            Some(path) => path,
            None => return,
        };
        self.email_folder = Some(path);
    }

    fn open_database_dialog(&mut self) {
        let default_path = "~/Desktop/";

        // FIXME: Not sure if this works
        #[cfg(target_os = "windows")]
        let default_path = "C:\\Users";

        let filename = rfd::FileDialog::new()
            .add_filter("sqlite", &["sqlite"])
            .set_directory(default_path)
            .save_file();

        let path = match filename {
            Some(path) => path,
            None => return,
        };
        self.database_path = Some(path);
    }

    fn draw_background(&mut self, ui: &mut egui::Ui, size: Vec2) {
        let painter = ui.painter();

        let division = 6.0;

        // paint stuff
        let rect_size = vec2(size.x / division, size.y / division);

        let offset = self.timer * 42.5;

        if offset > rect_size.x as f64 {
            self.timer = 0.0;
            self.offset_counter += 1;
        }

        // Reset the offset counter as we're going out of the size
        if (self.offset_counter as f32 * rect_size.x) > (size.x * 1.1) {
            self.offset_counter = 0;
        }

        // figure out the offset addition
        let add = self.offset_counter as i8; //(offset as f32 / rect_size.x) as i8;

        Self::draw_rectangles(
            painter,
            offset,
            division,
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
            division as usize,
        );

        let diff = ui.input().unstable_dt as f64;
        self.timer += diff;

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
                        Self::draw_segmentation(painter, rect, *rd);
                    }
                }
            }
        }
    }

    fn draw_segmentation(painter: &Painter, into: Rect, divisions: i8) {
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
            painter.rect_stroke(next, 0.0, Stroke::new(1.0, Color32::from_gray(70)));
            rect = next;
        }
    }
}
