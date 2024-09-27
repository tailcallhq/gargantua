use std::collections::{HashMap, HashSet};

use async_graphql::Positioned;
use async_graphql_parser::types as Q;
use blueprint::{Blueprint, Definition, FieldDefinition, ObjectTypeDefinition, Type};

use crate::{Argument, Directive, Field, QueryPlan, SelectionSet};

pub struct Builder {
    blueprint: Blueprint,
}

impl Builder {
    pub fn new(blueprint: Blueprint) -> Self {
        Self { blueprint }
    }

    pub fn build(&self, doc: &Q::ExecutableDocument) -> QueryPlan<String> {
        let paths = match &doc.operations {
            async_graphql_parser::types::DocumentOperations::Single(Positioned {
                node, ..
            }) => self.build_query_plan(&node.selection_set.node, "Query"),
            _ => todo!(),
        };
        println!(" paths: {:#?}", paths);
        QueryPlan::Sequence(vec![])
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

    fn get_field_def(&self, field_name: &str) -> &FieldDefinition {
        self.blueprint
            .definitions
            .iter()
            .find_map(|def| match def {
                Definition::Object(obj) => obj.fields.iter().find(|f| f.name == field_name),
                Definition::Interface(interface) => {
                    interface.fields.iter().find(|f| f.name == field_name)
                }
                _ => None,
            })
            .expect(&format!(
                "unable to find field definition for field : {}",
                field_name
            ))
    }

    fn get_type_def(&self, type_name: &str) -> &ObjectTypeDefinition {
        self.blueprint
            .definitions
            .iter()
            .find_map(|def| match def {
                Definition::Object(obj) => {
                    if obj.name == type_name {
                        Some(obj)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .expect(&format!(
                "unable to find type definition for type : {}",
                type_name
            ))
    }

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
                let field_def = self.get_field_def(&node.name.node);
                // let type_name = self.type_to_string(&field_def.of_type);

                // steps:
                // 1. if field have directive of join__field, then pick the subgraph from that
                //    directive.
                // 2. if field doens't have the join__field, then that pick belongs to local
                //    subgraph -> look at the join__type defination for subgraph.
                let subgraph = field_def
                    .join_fields
                    .first()
                    .and_then(|jf| jf.graph.as_ref())
                    .cloned()
                    .unwrap_or_else(|| {
                        let field_def = self.get_type_def(container_type);
                        field_def.join_types.first().cloned().unwrap().graph
                    });
                if let Some(fields) = paths.get_mut(subgraph.as_str()) {
                    fields.insert(node.name.node.to_string());
                } else {
                    let mut fields = HashSet::new();
                    fields.insert(node.name.node.to_string());
                    paths.insert(subgraph.as_str().to_string(), fields);
                }
                for selection_field in node.selection_set.node.items.iter() {
                    if let Q::Selection::Field(Positioned { node, .. }) = &selection_field.node {
                        let nested_field_def = self.get_field_def(&node.name.node);
                        let nested_field_type_name = self.type_to_string(&nested_field_def.of_type);
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
                            let field_def = self.get_type_def(&nested_field_type_name);
                            let join_type = field_def.join_types.first().cloned().unwrap();

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
                        let selection_sub_paths = self
                            .build_query_plan(&node.selection_set.node, &nested_field_type_name);
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

        paths
    }
}

impl SelectionSet<String> {
    fn from_gql_field(field: &Q::Field) -> Self {
        Self {
            fields: field
                .selection_set
                .node
                .items
                .iter()
                .filter_map(|s| {
                    if let Q::Selection::Field(f) = &s.node {
                        Some(Field::from_gql_field(&f.node))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

impl Field<String> {
    fn from_gql_field(field: &Q::Field) -> Self {
        Field {
            name: field.name.to_string(),
            selections: SelectionSet::from_gql_field(field),
            arguments: field
                .arguments
                .iter()
                .map(|(name, value)| Argument { name: name.to_string(), value: value.to_string() })
                .collect(),
            directives: field
                .directives
                .iter()
                .map(|d| Directive {
                    name: d.node.name.to_string(),
                    arguments: d
                        .node
                        .arguments
                        .iter()
                        .map(|(name, value)| Argument {
                            name: name.to_string(),
                            value: value.to_string(),
                        })
                        .collect(),
                })
                .collect(),
            is_hidden: false,
        }
    }
}

#[cfg(test)]
mod test {
    use blueprint::Blueprint;
    use valid::{Valid, Validator};

    use super::Builder;

    #[test]
    fn test() {
        // let query = "query  { topProducts { name reviews { score } reviews {
        // description } } }";
        let query = "query { topProducts { name reviews { score } reviews { description } } }";
        let p_query = async_graphql_parser::parse_query(query).unwrap();

        let graphql = resource::resource_str!("../examples/router.graphql");
        let document = async_graphql_parser::parse_schema(graphql).unwrap();
        let blueprint = Blueprint::parse(document).to_result().unwrap();

        // insta::assert_debug_snapshot!(blueprint);

        let builder = Builder::new(blueprint);
        let _ = builder.build(&p_query);

        // let node = match &p_query.operations {
        //     async_graphql_parser::types::DocumentOperations::Single(Positioned {
        //         node, ..
        //     }) => node.selection_set.node.clone(),
        //     _ => todo!(),
        // };

        // for selection in node.items.iter() {
        //     if let Selection::Field(Positioned { node, .. }) =
        // &selection.node {         let selection_field =
        // SelectionSet::from_gql_field(node);         println!("
        // [finder]: {:#?}", selection_field);         break;
        //     }
        // }
    }
}
