#[cfg(feature = "gui")]
fn main() {
    #[cfg(debug_assertions)]
    postsack::setup_tracing();

    postsack::gui::run_gui();
}

#[cfg(not(feature = "gui"))]
fn main() {
    println!("Gui not selected")
}
