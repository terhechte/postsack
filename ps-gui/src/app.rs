use std::marker::PhantomData;

use eframe::{
    egui::{self},
    epi::{self, App, Frame, Storage},
};

use super::app_state::StateUI;
use super::platform::Theme;
use super::textures::Textures;

use ps_core::DatabaseLike;

pub struct PostsackApp<Database: DatabaseLike> {
    state: StateUI,
    platform_custom_setup: bool,
    textures: Option<Textures>,
    _database: PhantomData<Database>,
}

impl<Database: DatabaseLike> PostsackApp<Database> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        let state = StateUI::new();
        PostsackApp {
            state,
            platform_custom_setup: false,
            textures: None,
            _database: PhantomData,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(config: ps_core::Config, total: usize) -> Self {
        let state = StateUI::new::<Database>(config, total);
        PostsackApp {
            state,
            platform_custom_setup: false,
            textures: None,
            _database: PhantomData,
        }
    }
}

impl<Database: DatabaseLike> App for PostsackApp<Database> {
    fn name(&self) -> &str {
        "Postsack"
    }

    fn setup(&mut self, ctx: &egui::CtxRef, frame: &mut Frame<'_>, _storage: Option<&dyn Storage>) {
        super::platform::setup(ctx, Theme::Dark);

        // Load textures
        self.textures = Some(Textures::populated(frame));
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        if !self.platform_custom_setup {
            self.platform_custom_setup = true;

            // Make the UI a bit bigger
            let pixels = ctx.pixels_per_point();
            ctx.set_pixels_per_point(pixels * 1.2);

            // If there is a platform error, display it
            if let Some(e) = super::platform::initial_update(ctx).err() {
                self.state = StateUI::error(e);
            }
        }

        self.state.update::<Database>(ctx, &self.textures);

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}
