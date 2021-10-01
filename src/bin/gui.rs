use gmaildb;

#[cfg(feature = "gui")]
fn main() {
    let config = gmaildb::make_config();
    gmaildb::gui::run_gui(config);
}

#[cfg(not(feature = "gui"))]
fn main() {
    println!("Gui not selected")
}
