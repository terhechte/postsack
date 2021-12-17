use ps_database::Database;
use ps_gui::{eframe, PostsackApp};

fn main() {
    #[cfg(debug_assertions)]
    ps_core::setup_tracing();

    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(PostsackApp::<Database>::new()), options);
}
