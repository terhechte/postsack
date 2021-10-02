use crate::database::query::{GroupByField, ValueField};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub year_filter: Option<usize>,
    pub domain_filter: String,
    pub search_stack: Vec<ValueField>,
    pub group_by_stack: Vec<GroupByField>,
}

impl State {
    pub fn new() -> Self {
        let mut state = State::default();
        state.group_by_stack.push(default_group_by_stack(0));
        state
    }
}

pub fn default_group_by_stack(index: usize) -> GroupByField {
    match index {
        0 => GroupByField::Year,
        1 => GroupByField::SenderDomain,
        2 => GroupByField::SenderLocalPart,
        _ => panic!(),
    }
}
