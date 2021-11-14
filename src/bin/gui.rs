#[cfg(feature = "gui")]
fn main() {
    postsack::setup_tracing();
    postsack::gui::run_gui();
}

#[cfg(not(feature = "gui"))]
fn main() {
    println!("Gui not selected")
}
