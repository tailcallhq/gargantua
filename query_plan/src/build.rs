use std::rc::Rc;
use std::vec;

use async_graphql::Positioned;
use async_graphql_parser::types::{self as Q, OperationDefinition};
use blueprint::{Graph, Index};
use valid::Valid;

use crate::error::Error;
use crate::{Field, SelectionSet};

pub struct Builder<A> {
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Default> Builder<A> {
    pub fn new() -> Self {
        Self { _phantom: std::marker::PhantomData }
    }

    // Multiple(HashMap<Name, Positioned<OperationDefinition>>),
    pub fn build(&self, operation: &Q::OperationDefinition) -> SelectionSet<A> {
        SelectionSet::from(&operation.selection_set.node)
    }
}

impl<A: Default> From<&Q::SelectionSet> for SelectionSet<A> {
    fn from(node: &Q::SelectionSet) -> SelectionSet<A> {
        let mut selection_set: SelectionSet<A> = SelectionSet::default();
        for selection in node.items.iter() {
            let inner_selection = &selection.node;
            match inner_selection {
                Q::Selection::Field(Positioned { node, .. }) => {
                    let field_name = node.name.node.as_str().to_string();
                    let field =
                        Field::new(field_name, SelectionSet::from(&node.selection_set.node));
                    selection_set.push(field);
                }
                Q::Selection::InlineFragment(Positioned { node, .. }) => {
                    todo!()
                }
                Q::Selection::FragmentSpread(Positioned { node, .. }) => {
                    todo!()
                }
            }
        }
        selection_set
    }
}

#[cfg(test)]
mod test {
    use blueprint::Blueprint;
    use insta::assert_debug_snapshot;
    use resource::resource_str;
    use serde_json::Value;

    use crate::{Builder, QueryPlan, SelectionSet};

    #[test]
    fn test() {
        // Query
        let query = "query { topProducts { name reviews { score } reviews { description } } }";
        let p_query = async_graphql_parser::parse_query(query).unwrap();
        let op = match p_query.operations {
            async_graphql_parser::types::DocumentOperations::Single(op) => op.node,
            _ => todo!(),
        };
        let selection_set: SelectionSet<String> = Builder::new().build(&op);
        assert_debug_snapshot!(selection_set);
    }
}
