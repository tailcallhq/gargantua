use std::collections::{HashMap, HashSet};

use async_graphql::Positioned;
use async_graphql_parser::types as Q;
use blueprint::{
    Blueprint, Definition, FieldDefinition, Index, ObjectTypeDefinition, QueryField, Type,
};
use valid::Valid;

use crate::QueryPlan;

pub struct Builder<A> {
    index: Index,
    _phantom: std::marker::PhantomData<A>,
}

impl<A> Builder<A> {
    pub fn new(index: Index) -> Self {
        Self { index, _phantom: std::marker::PhantomData }
    }

    pub fn build(&self, doc: &Q::ExecutableDocument) -> Valid<QueryPlan<A>, String> {
        let paths = match &doc.operations {
            async_graphql_parser::types::DocumentOperations::Single(Positioned {
                node, ..
            }) => self.build_query_plan(&node.selection_set.node, "Query"),
            _ => todo!(),
        };
        Valid::succeed(QueryPlan::Sequence(vec![]))
    }

    fn type_to_string(&self, type_: &Type) -> String {
        match type_ {
            Type::Named { name, required } => {
                if *required {
                    format!("{}", name)
                } else {
                    name.clone()
                }
            }
            Type::List { of_type, non_null } => {
                let inner = self.type_to_string(of_type);
                if *non_null {
                    format!("{}", inner)
                } else {
                    format!("{}", inner)
                }
            }
        }
    }

    // fn get_field_def(&self, type_name: &str, field_name: &str) -> &FieldDefinition {
    //     self.index.get_field(type_name, field_name)
    // }

    // fn get_type_def(&self, type_name: &str) -> &ObjectTypeDefinition {
    //     self.index
    //         .definitions
    //         .iter()
    //         .find_map(|def| match def {
    //             Definition::Object(obj) => {
    //                 if obj.name == type_name {
    //                     Some(obj)
    //                 } else {
    //                     None
    //                 }
    //             }
    //             _ => None,
    //         })
    //         .expect(&format!(
    //             "unable to find type definition for type : {}",
    //             type_name
    //         ))
    // }

    fn build_query_plan(
        &self,
        selections: &Q::SelectionSet,
        container_type: &str,
    ) -> HashMap<String, HashSet<String>> {
        let mut paths: HashMap<String, HashSet<String>> = HashMap::new();

        for selection in selections.items.iter() {
            if let Q::Selection::Field(Positioned { node, .. }) = &selection.node {
                // 1. fetch operation
                // let plan = SimplePlan::Fetch { service: "product-svc".into(), query: "query {
                // topProducts { name __typename upc} }".into() };

                // Find the field definition in the blueprint
                if let Some(query_field) = self.index.get_field(type_name, field_name) {
                    let field_def = match query_field {
                        QueryField::Field(query_field) => query_field.0,
                        _ => todo!(),
                    };

                    let type_name = self.type_to_string(&field_def.of_type);

                    // steps:
                    // 1. if field have directive of join__field, then pick the subgraph from that
                    //    directive.
                    // 2. if field doesn't have the join__field, then that pick belongs to local
                    //    subgraph -> look at the join__type definition for subgraph.
                    let subgraph = field_def
                        .join_fields
                        .first()
                        .and_then(|jf| jf.graph.as_ref())
                        .cloned()
                        .unwrap_or_else(|| {
                            match self.index.get_type(container_type) {
                                Some(type_info) => type_info.0,
                                None => todo!("raise error!")
                            }
                        });
                    if let Some(fields) = paths.get_mut(subgraph.as_str()) {
                        fields.insert(node.name.node.to_string());
                    } else {
                        let mut fields = HashSet::new();
                        fields.insert(node.name.node.to_string());
                        paths.insert(subgraph.as_str().to_string(), fields);
                    }
                    for selection_field in node.selection_set.node.items.iter() {
                        if let Q::Selection::Field(Positioned { node, .. }) = &selection_field.node
                        {
                            let nested_field_def = match self.index.get_field(&type_name, &node.name.node) {
                                Some(QueryField::Field(field)) => field.0,
                                _ => todo!("fix it later."),
                            };
                            let nested_field_type_name =
                                self.type_to_string(&nested_field_def.of_type);
                            let nested_field_subgraph = nested_field_def
                                .join_fields
                                .first()
                                .and_then(|jf| jf.graph.as_ref())
                                .cloned()
                                .unwrap_or_else(|| {
                                    field_def
                                        .join_fields
                                        .first()
                                        .cloned()
                                        .unwrap()
                                        .graph
                                        .unwrap()
                                });

                            if nested_field_subgraph == subgraph {
                                if let Some(fields) = paths.get_mut(subgraph.as_str()) {
                                    fields.insert(node.name.node.to_string());
                                } else {
                                    let mut fields = HashSet::new();
                                    fields.insert(node.name.node.to_string());
                                    paths.insert(subgraph.as_str().to_string(), fields);
                                }
                            } else {
                                // before checking on type, we should check for the join on field.
                                // let field_def = self.get_type_def(&nested_field_type_name);
                                // match self.index.get_type&nested_field_type_name) {
                                //     Some(type_info) => type_name
                                // }
                                let type_def = match self.index.get_type(&nested_field_type_name) {
                                    Some(type_info) => type_info.0,
                                    None => todo!("raise error")
                                };

                                let join_type = type_def.join_types.first().cloned().unwrap();

                                let mut new_fields = HashSet::new();
                                new_fields.insert("__typename".to_string());

                                if join_type.key.is_some() {
                                    new_fields.insert(join_type.key.unwrap());
                                }

                                if let Some(fields) = paths.get_mut(subgraph.as_str()) {
                                    fields.extend(new_fields);
                                } else {
                                    paths.insert(subgraph.as_str().to_string(), new_fields);
                                }
                            }

                            // if sub-field has it's own selection set then -> explore that.
                            // TODO: i think these sub-fields plans can be executed in parallel manner
                            // but verify that with various edge cases.
                            // nesting operations must be handled sequentially with respect to its
                            // parent.
                            let selection_sub_paths = self.build_query_plan(
                                &node.selection_set.node,
                                &nested_field_type_name,
                            );
                            for (subgraph, selection_set) in selection_sub_paths {
                                if let Some(fields) = paths.get_mut(subgraph.as_str()) {
                                    fields.extend(selection_set);
                                } else {
                                    paths.insert(subgraph, selection_set);
                                }
                            }
                        }
                    }
                }
            }
        }

        paths
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
