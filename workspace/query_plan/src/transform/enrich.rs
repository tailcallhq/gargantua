use std::marker::PhantomData;

use blueprint::{ Index, QueryField };
use valid::{ Transform, Valid, Validator };

use crate::{ QueryOperation, QueryPlan, SelectionSet };

pub struct Enrich<Value> {
    index: Index,
    root_type: String,
    _marker: PhantomData<Value>,
}

impl<Value: Clone> Enrich<Value> {
    pub fn new(index: Index, root_type: String) -> Self {
        Self { index, root_type, _marker: PhantomData }
    }

    fn iter_sel(
        &self,
        selection: SelectionSet<Value>,
        container_type: &str
    ) -> Valid<SelectionSet<Value>, String> {
        // this field belongs to container_type, so we if want to get this field
        let type_def = match self.index.get_object_type_definition(container_type) {
            Some(type_def) => type_def,
            None => {
                return Valid::fail(
                    format!("type definition not found for type '{}' ", container_type)
                );
            }
        };

        Valid::from_iter(selection.into_vec().into_iter(), |field| {
            let field_def = match self.index.get_field(container_type, &field.name) {
                Some(QueryField::Field((def, _))) => def,
                _ => {
                    return Valid::fail(
                        format!(
                            "field definition not found for field '{}' in type '{}' ",
                            field.name,
                            container_type
                        )
                    );
                }
            };

            let field = if field_def.join_fields.is_empty() {
                // if field doesn't have @join__field directive, then
                // we need to figure out from where this field can be queried

                // 1. this field can be queried form the @join__type -> wherein the key is same
                //    as this field.
                // 2. this field can be queried from the @join__type directive's graph where key
                //    is none.
                let graphs = type_def.join_types
                    .iter()
                    .filter_map(|jt| {
                        if jt.key.is_none() || jt.key.as_ref().map_or(false, |k| k == &field.name) {
                            Some(jt.graph.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                field.graph(graphs)
            } else {
                field.join_field(field_def.join_fields.clone())
            };

            let type_name = field_def.of_type.as_type_str();
            let selection = field.selections.clone();
            self.iter_sel(selection, &type_name).map(|selection_set|
                field.selections(selection_set)
            )
        }).map(|fields| SelectionSet::new(fields))
    }

    fn iter(
        &self,
        query: QueryPlan<Value>,
        container_type: &str
    ) -> Valid<QueryPlan<Value>, String> {
        match query {
            QueryPlan::Fetch { service, query, representations, type_name } =>
                self
                    .iter_sel(query.selection_set, container_type)
                    .map(|selection_set| QueryPlan::Fetch {
                        service,
                        query: QueryOperation { selection_set },
                        representations,
                        type_name,
                    }),
            QueryPlan::Flatten { select, plan } =>
                self
                    .iter(*plan, container_type)
                    .map(|plan| QueryPlan::Flatten { select, plan: Box::new(plan) }),

            QueryPlan::Parallel(plans) => {
                Valid::from_iter(plans, |plan| self.iter(plan, container_type)).map(|plans|
                    QueryPlan::Parallel(plans)
                )
            }

            QueryPlan::Sequence(plans) => {
                Valid::from_iter(plans, |plan| self.iter(plan, container_type)).map(|plans|
                    QueryPlan::Sequence(plans)
                )
            }
        }
    }
}

impl<Value: Clone> Transform for Enrich<Value> {
    type Value = QueryPlan<Value>;
    type Error = String;

    fn transform(&self, value: Self::Value) -> valid::Valid<Self::Value, Self::Error> {
        self.iter(value, &self.root_type)
    }
}

#[cfg(test)]
mod test {
    use blueprint::Blueprint;

    use super::*;

    fn setup(graphql: &str) -> Index {
        let document = async_graphql_parser::parse_schema(graphql).unwrap();

        Blueprint::parse_doc(document).to_index()
    }

    #[test]
    fn test_enricher_supergraph_1() {
        let query = "query { topProducts { productName: name reviews { body } reviews { id } } }";
        let index = setup(include_str!("../../../blueprint/src/fixtures/router.graphql"));
        let doc = async_graphql_parser::parse_query(query).unwrap();

        let qp = QueryPlan::try_new(&query).unwrap();

        let enriched_selection_set = Enrich::new(index, "Query".to_string())
            .transform(qp)
            .to_result()
            .unwrap();

        insta::assert_debug_snapshot!(enriched_selection_set)
    }
}
