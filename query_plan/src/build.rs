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
}

#[cfg(test)]
mod test {
    use super::Builder;
    use crate::QueryPlan;
    use blueprint::Blueprint;
    use insta::assert_debug_snapshot;
    use valid::{Valid, Validator};

    #[test]
    fn test() {
        // Blueprint
        let graphql = resource::resource_str!("../examples/router.graphql");
        let blueprint = Blueprint::parse(graphql).to_result().unwrap();

        // Query
        let query = "query { topProducts { name reviews { score } reviews { description } } }";
        let plan = QueryPlan::try_new(query.to_string(), blueprint.to_index()).unwrap();

        assert_debug_snapshot!(plan);
    }
}
