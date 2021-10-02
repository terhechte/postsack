use crate::cluster_engine::IntoRequest;
use crate::database::query::{Filter, ValueField};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub year_filter: Option<usize>,
    pub domain_filter: String,
}

impl State {
    pub fn new() -> Self {
        State::default()
    }
}

impl IntoRequest for State {
    fn into_filters(&self) -> Vec<crate::database::query::Filter> {
        let mut filters = Vec::new();

        if !self.domain_filter.is_empty() {
            filters.push(Filter::Like(ValueField::SenderDomain(
                self.domain_filter.clone().into(),
            )));
        }
        if let Some(n) = self.year_filter {
            filters.push(Filter::Is(ValueField::Year(n)));
        }

        filters
    }
}
