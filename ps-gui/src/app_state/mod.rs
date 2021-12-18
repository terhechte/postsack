mod error;
mod import;
mod main;
mod startup;

use std::path::PathBuf;

pub use super::textures::Textures;
use eframe::egui::{self};
pub use error::ErrorUI;
use eyre::Report;
pub use import::ImporterUI;
pub use main::{MainUI, UIState};
pub use startup::StartupUI;

use ps_core::{Config, DatabaseLike, FormatType};

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
        total: usize,
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

pub enum StateUI {
    Startup(startup::StartupUI),
    Import(import::ImporterUI),
    Main(main::MainUI),
    Error(error::ErrorUI),
}

pub trait StateUIVariant {
    fn update_panel(&mut self, ctx: &egui::CtxRef, textures: &Option<Textures>) -> StateUIAction;
}

impl StateUI {
    /// Create an error state
    pub fn error(report: Report) -> StateUI {
        StateUI::Error(error::ErrorUI::new(report, None))
    }

    /// This proxies the `update` call to the individual calls in
    /// the `app_state` types
    pub fn update<Database: DatabaseLike>(
        &mut self,
        ctx: &egui::CtxRef,
        textures: &Option<Textures>,
    ) {
        let response = match self {
            StateUI::Startup(panel) => panel.update_panel(ctx, textures),
            StateUI::Import(panel) => panel.update_panel(ctx, textures),
            StateUI::Main(panel) => panel.update_panel(ctx, textures),
            StateUI::Error(panel) => panel.update_panel(ctx, textures),
        };
        match response {
            StateUIAction::CreateDatabase {
                database_path,
                emails_folder_path,
                sender_emails,
                format,
            } => {
                *self = self.create_database::<Database>(
                    database_path,
                    emails_folder_path,
                    sender_emails,
                    format,
                )
            }
            StateUIAction::OpenDatabase { database_path } => {
                *self = self.open_database::<Database>(database_path)
            }
            StateUIAction::ImportDone { config, total } => {
                *self = match main::MainUI::new::<Database>(config.clone(), total) {
                    Ok(n) => StateUI::Main(n),
                    Err(e) => StateUI::Error(ErrorUI::new(e, Some(config))),
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
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> StateUI {
        StateUI::Startup(startup::StartupUI::default())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new<Database: DatabaseLike>(config: Config, total: usize) -> StateUI {
        StateUI::Main(main::MainUI::new::<Database>(config, total).unwrap())
    }

    pub fn create_database<Database: DatabaseLike>(
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

        let database = match Database::new(&config.database_path) {
            Ok(config) => config,
            Err(report) => return StateUI::Error(error::ErrorUI::new(report, None)),
        };

        self.importer_with_config(config, database)
    }

    pub fn open_database<Database: DatabaseLike>(&mut self, database_path: PathBuf) -> StateUI {
        let config = match Database::config(&database_path) {
            Ok(config) => config,
            Err(report) => return StateUI::Error(error::ErrorUI::new(report, None)),
        };

        let total = match Database::new(&database_path).and_then(|db| db.total_mails()) {
            Ok(config) => config,
            Err(report) => return StateUI::Error(error::ErrorUI::new(report, None)),
        };

        match main::MainUI::new::<Database>(config.clone(), total) {
            Ok(n) => StateUI::Main(n),
            Err(e) => StateUI::Error(ErrorUI::new(e, Some(config))),
        }
    }

    fn importer_with_config<Database: DatabaseLike>(
        &self,
        config: Config,
        database: Database,
    ) -> StateUI {
        let importer = match import::ImporterUI::new(config.clone(), database) {
            Ok(n) => n,
            Err(e) => {
                return StateUI::Error(error::ErrorUI::new(e, Some(config)));
            }
        };

        StateUI::Import(importer)
    }
}
