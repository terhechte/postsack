mod error;
mod import;
mod main;
mod startup;

use std::path::PathBuf;

use eframe::egui::{self, Widget};
pub use error::ErrorUI;
use eyre::{Report, Result};
pub use import::ImporterUI;
pub use main::{MainUI, UIState};
pub use startup::StartupUI;

use crate::types::{Config, FormatType};

pub enum StateUIAction {
    CreateDatabase {
        database_path: Option<PathBuf>,
        emails_folder_path: PathBuf,
        sender_emails: Vec<String>,
        format: FormatType,
    },
    OpenDatabase {
        database_path: PathBuf,
    },
    ImportDone {
        config: Config,
    },
    Close {
        config: Config,
    },
    Error {
        report: Report,
        config: Config,
    },
    Nothing,
}

// FIXME: Removve
// pub fn make_temporary_ui_config() -> crate::types::Config {
//     crate::types::Config::new(
//         "./db6.sql",
//         "",
//         vec!["terhechte@me.com".to_string()],
//         crate::types::FormatType::AppleMail,
//     )
// }

// pub enum MainApp {
//     Startup { panel: StartupUI },
//     Import { panel: ImporterUI },
//     Main { panel: MainUI },
//     Error { panel: ErrorUI },
// }

/// This defines the state machine switches between the `MainApp`
/// states.
/// FIXME: I'm not particularly happy with this abstraction right now.
// impl MainApp {
//     /// An Error state can always happen
//     pub fn error(report: eyre::Report) -> MainApp {
//         MainApp::Error {
//             panel: ErrorUI(report),
//         }
//     }
//     // pub fn import(startup: &) -> MainApp {

//     // }
// }
pub enum StateUI {
    Startup(startup::StartupUI),
    Import(import::ImporterUI),
    Main(main::MainUI),
    Error(error::ErrorUI),
}

pub trait StateUIVariant {
    fn update_panel(&mut self, ctx: &egui::CtxRef) -> StateUIAction;
}

impl StateUI {
    /// This proxies the `update` call to the individual calls in
    /// the `app_state` types
    pub fn update(&mut self, ctx: &egui::CtxRef) {
        let response = match self {
            StateUI::Startup(panel) => panel.update_panel(ctx),
            StateUI::Import(panel) => panel.update_panel(ctx),
            StateUI::Main(panel) => panel.update_panel(ctx),
            StateUI::Error(panel) => panel.update_panel(ctx),
        };
        match response {
            StateUIAction::CreateDatabase {
                database_path,
                emails_folder_path,
                sender_emails,
                format,
            } => {
                *self =
                    self.create_database(database_path, emails_folder_path, sender_emails, format)
            }
            StateUIAction::OpenDatabase { database_path } => {
                *self = self.open_database(database_path)
            }
            StateUIAction::ImportDone { config } => {
                *self = match main::MainUI::new(config.clone()) {
                    Ok(n) => StateUI::Main(n),
                    Err(e) => StateUI::Error(ErrorUI::new(e, Some(config.clone()))),
                };
            }
            StateUIAction::Close { config } => {
                *self = StateUI::Startup(StartupUI::from_config(config));
            }
            StateUIAction::Error { report, config } => {
                *self = StateUI::Error(error::ErrorUI::new(report, Some(config)))
            }
            StateUIAction::Nothing => (),
        }
    }
}

impl StateUI {
    pub fn new() -> StateUI {
        StateUI::Startup(startup::StartupUI::default())
    }

    pub fn create_database(
        &self,
        database_path: Option<PathBuf>,
        emails_folder_path: PathBuf,
        sender_emails: Vec<String>,
        format: FormatType,
    ) -> StateUI {
        let config = match Config::new(database_path, emails_folder_path, sender_emails, format) {
            Ok(n) => n,
            Err(e) => {
                return StateUI::Error(error::ErrorUI::new(e, None));
            }
        };

        let importer = match import::ImporterUI::new(config.clone()) {
            Ok(n) => n,
            Err(e) => {
                return StateUI::Error(error::ErrorUI::new(e, Some(config.clone())));
            }
        };

        return StateUI::Import(importer);
    }

    pub fn open_database(&mut self, database_path: PathBuf) -> StateUI {
        // FIXME: the database needs to be opened in order to figure
        // out whether it is a correct DB, before we can head on
        todo!()
    }
}
