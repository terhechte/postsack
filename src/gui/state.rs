use crate::database::query::{GroupByField, ValueField};

pub type GroupByFieldIndex = usize;

// FIXME: Use strum or one of the enum to string crates
const DEFAULT_GROUP_BY_FIELDS: &[GroupByField] = {
    use GroupByField::*;
    &[
        SenderDomain,
        SenderLocalPart,
        SenderName,
        Year,
        Month,
        Day,
        ToGroup,
        ToName,
        ToAddress,
        IsReply,
        IsSend,
    ]
};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub year_filter: Option<usize>,
    pub domain_filter: String,
    pub search_stack: Vec<ValueField>,
    pub group_by_stack: Vec<usize>,
}

impl State {
    pub fn group_by_field_for(index: usize) -> GroupByField {
        DEFAULT_GROUP_BY_FIELDS[index]
    }

    pub fn all_group_by_fields() -> Vec<GroupByField> {
        DEFAULT_GROUP_BY_FIELDS.into()
    }

    pub fn new() -> Self {
        let mut state = State::default();
        state.group_by_stack.push(default_group_by_stack(0));
        state
    }

    pub fn group_by_fields(&self) -> Vec<GroupByField> {
        self.group_by_stack
            .iter()
            .map(|e| Self::group_by_field_for(*e))
            .collect()
    }
}

/// Return the default group by fields index for each stack entry
pub fn default_group_by_stack(index: usize) -> usize {
    let f = match index {
        0 => GroupByField::Year,
        1 => GroupByField::SenderDomain,
        2 => GroupByField::SenderLocalPart,
        3 => GroupByField::Month,
        4 => GroupByField::Day,
        _ => panic!(),
    };
    DEFAULT_GROUP_BY_FIELDS
        .iter()
        .position(|e| e == &f)
        .unwrap()
}
