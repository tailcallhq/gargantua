use std::vec;

use async_graphql_parser::types as Q;
use blueprint::{Graph, Index};
use valid::Valid;

use crate::{QueryPlan, SelectionSet, TypeName};

pub struct Builder<A> {
    index: Index,
    _phantom: std::marker::PhantomData<A>,
}

impl<A> Builder<A> {
    pub fn new(index: Index) -> Self {
        Self { index, _phantom: std::marker::PhantomData }
    }

    pub fn build(&self, doc: &Q::ExecutableDocument) -> Valid<QueryPlan<A>, String> {
        Valid::succeed(QueryPlan::fetch(
            Graph::new("Product"),
            TypeName::new("Query"),
            SelectionSet { fields: vec![] },
        ))
    }

    fn build_operation(&self, operation: &Q::OperationDefinition) -> Valid<QueryPlan<A>, String> {
        match operation.ty {
            Q::OperationType::Query => self.build_query(operation),
            Q::OperationType::Mutation => todo!(),
            Q::OperationType::Subscription => todo!(),
        }
    }

    fn build_query(&self, operation: &Q::OperationDefinition) -> Valid<QueryPlan<A>, String> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use blueprint::Blueprint;
    use insta::assert_debug_snapshot;
    use resource::resource_str;

    use crate::QueryPlan;

    #[test]
    fn test() {
        // Blueprint
        let graphql = resource_str!("../fixtures/router.graphql");
        let blueprint = Blueprint::parse(graphql.as_ref()).unwrap();

        // Query
        let query = "query { topProducts { name reviews { score } reviews { description } } }";
        let plan: QueryPlan<()> =
            QueryPlan::try_new(query.to_string(), blueprint.to_index()).unwrap();

        assert_debug_snapshot!(plan);
    }
}
