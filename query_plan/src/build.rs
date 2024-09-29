use std::{rc::Rc, vec};

use async_graphql_parser::types as Q;
use blueprint::{Graph, Index};
use valid::Valid;

use crate::SelectionSet;

pub struct Builder<A> {
    index: Rc<Index>,
    _phantom: std::marker::PhantomData<A>,
}

impl<A> Builder<A> {
    pub fn new(index: Rc<Index>) -> Self {
        Self { index, _phantom: std::marker::PhantomData }
    }

    pub fn build(&self, doc: &Q::ExecutableDocument) -> Valid<SelectionSet<A>, String> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use blueprint::Blueprint;
    use insta::assert_debug_snapshot;
    use resource::resource_str;
    use serde_json::Value;

    use crate::{QueryPlan, SelectionSet};

    #[test]
    fn test() {
        // Blueprint
        let graphql = resource_str!("../fixtures/router.graphql");
        let blueprint = Blueprint::parse(graphql.as_ref()).unwrap();

        // Query
        let query = "query { topProducts { name reviews { score } reviews { description } } }";
        let plan: SelectionSet<Value> =
            SelectionSet::try_new(query.to_string(), &blueprint.to_index()).unwrap();

        assert_debug_snapshot!(plan);
    }
}
