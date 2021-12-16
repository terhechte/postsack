//! The startup form to configure what and how to import
use eframe::egui::epaint::Shadow;
use eframe::egui::{self, vec2, Color32, Pos2, Rect, Response, Stroke, TextStyle, Vec2};

use std::path::PathBuf;
use std::str::FromStr;

use super::super::platform::platform_colors;
use super::super::widgets::background::{shadow_background, AnimatedBackground};
use super::Textures;
use super::{StateUIAction, StateUIVariant};
use ps_core::{Config, FormatType};

#[derive(Default)]
pub struct StartupUI {
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
    /// Potential error message to display to the user
    error_message: Option<String>,
    /// The result of the actions
    action: Option<StateUIAction>,
}

impl StartupUI {
    pub fn from_config(config: Config) -> Self {
        let emails = if !config.sender_emails.is_empty() {
            let mails: Vec<String> = config.sender_emails.iter().map(|e| e.to_owned()).collect();
            Some(mails.join(", "))
        } else {
            None
        };
        // Only for persistent config do we re-populate the database path
        // otherwise it would hsow the temporary path
        let (save_to_disk, database_path) = match config.persistent {
            true => (true, Some(config.database_path)),
            false => (false, None),
        };
        Self {
            format: config.format,
            email_folder: Some(config.emails_folder_path),
            database_path,
            save_to_disk,
            email_address: emails,
            ..Default::default()
        }
    }
}

impl StateUIVariant for StartupUI {
    fn update_panel(
        &mut self,
        ctx: &egui::CtxRef,
        _textures: &Option<Textures>,
    ) -> super::StateUIAction {
        egui::CentralPanel::default()
            .frame(egui::containers::Frame::none().fill(Color32::BLACK))
            .show(ctx, |ui| {
                ui.add(|ui: &mut egui::Ui| self.ui(ui));
            });
        // If we generated an action above, return it
        self.action.take().unwrap_or(StateUIAction::Nothing)
    }
}

impl StartupUI {
    /// Separated to have a less stuff happening
    fn ui(&mut self, ui: &mut egui::Ui) -> Response {
        let available = ui.available_size();

        AnimatedBackground {
            divisions: 6,
            animate_progress: None,
            timer: &mut self.timer,
            offset_counter: &mut self.offset_counter,
        }
        .draw_background(ui, available);

        // I did not find an easy solution to center a frame in
        // on the vertical and horizontal position.
        // I tried `ui.centered_and_justified`,
        // `ui.allocate_exact_size`
        // `ui.allocate_with_layout`
        // and variations. This, at least, worked.
        let desired_size = egui::vec2(450.0, 370.0);

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
            colors.window_background,
            Stroke::new(1.0, Color32::from_gray(90)),
            12.0,
            Shadow::big_dark(),
        );

        let visuals = ui.visuals();
        let hyperlink_color = visuals.hyperlink_color;

        // placeholder text
        let mut txt = self
            .email_address
            .clone()
            .unwrap_or_else(|| "john@example.org".to_string());

        let response = ui.allocate_ui_at_rect(center, |ui| {
            // We use a grid as that gives us more spacing opportunities
            egui::Grid::new("filter_grid")
                .spacing(vec2(15.0, 12.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::widgets::Label::new("Choose Import Format:")
                            .text_color(colors.text_primary)
                            .text_style(TextStyle::Body),
                    );
                    ui.end_row();

                    self.format_selection(ui, center.width() * 0.7);
                    ui.end_row();

                    ui.add(
                        egui::widgets::Label::new("Email Folder:")
                            .text_color(platform_colors().text_primary)
                            .text_style(TextStyle::Body),
                    );
                    ui.end_row();

                    ui.horizontal(|ui| {
                        if ui.button("Browse...").clicked() {
                            self.open_email_folder_dialog()
                        }
                        if self.format == FormatType::AppleMail && ui.button("or Mail.app default folder").clicked(){
                            self.email_folder = ps_importer::default_path(&self.format);
                        }
                    });
                    ui.end_row();
                    if let Some(n) = self.email_folder.as_ref() {
                        let label = egui::widgets::Label::new(format!("{}", n.display()))
                             .text_color(hyperlink_color);
                        ui.add(label)
                            .on_hover_text(format!("{}", self.email_folder.as_ref().unwrap().display()));
                    }
                    ui.end_row();

                    ui.add(
                        egui::widgets::Label::new("Your Email Address:").text_color(colors.text_primary),
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
                            .text_color(platform_colors().text_secondary)
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
                                self.save_database_dialog()
                            }
                            if let Some(Some(Some(name))) = self.database_path.as_ref().map(|e| e.file_name().map(|e| e.to_str().map(|e| e.to_string()))) {
                                let label = egui::widgets::Label::new(name)
                                    .text_color(hyperlink_color);
                                ui.add(label)
                                    .on_hover_text(format!("{}", self.database_path.as_ref().unwrap().display()));
                            }
                        });
                    }
                    ui.end_row();

                    let button_size1: Vec2 = ((center.width() / 2.0) - 25.0, 25.0).into();
                    let button_size2: Vec2 = ((center.width() / 2.0) - 25.0, 25.0).into();
                    ui.horizontal(|ui| {
                        let enabled = {
                                // if we have an email folder,
                                // and - if we want to save to disk -
                                // if we have a database path
                                self.email_folder.is_some() &&
                                (self.save_to_disk == self.database_path.is_some())
                        };
                        ui.add_enabled_ui(enabled, |ui| {
                            let response = ui.add_sized(
                                button_size1,
                                egui::Button::new("Start").text_color(colors.text_primary),
                            );
                            if response.clicked() {
                                self.action_start();
                            }
                        });
                        let response = ui.add_sized(button_size2, egui::Button::new("Or Open Database"));
                        if response.clicked() {
                            self.action_open_database();
                        }
                    });
                    ui.end_row();
                    if let Some(ref e) = self.error_message {
                        let r = Color32::from_rgb(255, 0, 0);
                        ui.colored_label(r, e);
                    }
                });
        });

        response.response
    }
}

impl StartupUI {
    fn action_start(&mut self) {
        let email = match &self.email_folder {
            Some(n) => n.clone(),
            _ => return,
        };

        // Split by comma, remove whitespace
        let emails: Vec<String> = self
            .email_address
            .iter()
            .map(|e| e.split(',').map(|e| e.trim().to_string()).collect())
            .collect();

        if !email.exists() {
            self.error_message = Some("Email folder doesn't exist".into());
            return;
        }

        if self.save_to_disk && self.database_path.is_none() {
            self.error_message = Some("Please select a database folder".into());
            return;
        }

        self.action = Some(StateUIAction::CreateDatabase {
            database_path: self.database_path.clone(),
            emails_folder_path: email,
            sender_emails: emails,
            format: self.format,
        });
    }

    fn action_open_database(&mut self) {
        let path = match self.open_database_dialog() {
            Some(n) => n,
            None => return,
        };
        self.action = Some(StateUIAction::OpenDatabase {
            database_path: path,
        });
    }

    fn format_selection(&mut self, ui: &mut egui::Ui, width: f32) {
        let mut selected = self.format;
        egui::ComboBox::from_id_source("mailbox_type_combobox")
            .width(width)
            .selected_text(selected.name())
            .show_ui(ui, |ui| {
                for format in FormatType::all_cases() {
                    ui.selectable_value(&mut selected, format, format.name());
                }
            });
        self.format = selected;
    }

    fn open_email_folder_dialog(&mut self) {
        let fallback = shellexpand::tilde("~/");
        let default_path = ps_importer::default_path(&self.format)
            .unwrap_or_else(|| std::path::Path::new(&fallback.to_string()).to_path_buf());

        let folder = match tinyfiledialogs::select_folder_dialog(
            "Select folder",
            default_path.to_str().unwrap_or(""),
        ) {
            Some(result) => PathBuf::from_str(&result).ok(),
            None => return,
        };

        let path = match folder {
            Some(path) => path,
            None => return,
        };
        self.email_folder = Some(path);
    }

    fn save_database_dialog(&mut self) {
        let default_path = "~/Desktop/";

        // FIXME: Not sure if this works
        #[cfg(target_os = "windows")]
        let default_path = "C:\\Users";

        let fallback = shellexpand::tilde(default_path).to_string();

        let filename = match tinyfiledialogs::save_file_dialog("Select output file", &fallback) {
            Some(result) => PathBuf::from_str(&result).ok(),
            None => return,
        };

        let path = match filename {
            Some(path) => path,
            None => return,
        };

        self.database_path = Some(path)
    }

    fn open_database_dialog(&mut self) -> Option<PathBuf> {
        let default_path = "~/Desktop/";

        // FIXME: Not sure if this works
        #[cfg(target_os = "windows")]
        let default_path = "C:\\Users";

        let fallback = shellexpand::tilde(default_path).to_string();

        let filename = match tinyfiledialogs::open_file_dialog(
            "SQlite Database",
            &fallback,
            Some((&["*.sqlite"], "SQLite")),
        ) {
            Some(n) => PathBuf::from_str(&n).ok(),
            None => return None,
        };

        let path = match filename {
            Some(path) => path,
            None => return None,
        };

        Some(path)
    }
}
