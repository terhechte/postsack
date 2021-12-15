use crate::database::query_result::QueryRow;

/// Is a individual row/item being loaded or already loaded.
/// Used in a cache to improve the loading of data for the UI.
pub enum LoadingState {
    Loaded(QueryRow),
    Loading,
}
