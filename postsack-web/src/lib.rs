#[cfg(target_arch = "wasm32")]
use ps_core::{Config, FormatType};

#[cfg(target_arch = "wasm32")]
use ps_gui::{eframe, PostsackApp};

mod database;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

/// Call this once from the HTML.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    use database::FakeDatabase;

    let format = FormatType::AppleMail;
    let config = Config::new(None, "", vec!["test@gmail.com".to_owned()], format).unwrap();

    let app = PostsackApp::<database::FakeDatabase>::new(config, FakeDatabase::total_item_count());
    eframe::start_web(canvas_id, Box::new(app))
}
