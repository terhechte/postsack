#[derive(Debug, Clone)]
pub struct State {
    pub year_filter: Option<usize>,
    pub domain_filter: Option<String>,
}
