use ps_database::Database;

fn main() {
    #[cfg(debug_assertions)]
    ps_core::setup_tracing();

    ps_gui::run_ui::<Database>();
}
