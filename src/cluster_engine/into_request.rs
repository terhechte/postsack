use crate::database::query::Filter;

pub trait IntoRequest {
    fn into_filters(&self) -> Vec<Filter>;
}
