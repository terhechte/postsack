#[cfg(feature = "gui")]
fn main() {
    gmaildb::gui::run_gui();
}

#[cfg(not(feature = "gui"))]
fn main() {
    println!("Gui not selected")
}
