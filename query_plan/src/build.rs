use crate::SelectionSet;
use async_graphql_parser::types::{self as Q};

pub struct Builder<A> {
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Default> Builder<A> {
    pub fn new() -> Self {
        Self { _phantom: std::marker::PhantomData }
    }

    pub fn build(&self, operation: &Q::OperationDefinition) -> SelectionSet<A> {
        SelectionSet::from(&operation.selection_set.node)
    }
}
