use async_graphql_value::Value;
use blueprint::{Index, QueryField};
use valid::{Transform, Valid, Validator};

use crate::SelectionSet;

struct Enrich(Index);

impl Enrich {
    pub fn new(index: Index) -> Self {
        Self(index)
    }

    fn enrich_information(
        &self,
        selection: SelectionSet<Value>,
        container_type: &str,
    ) -> Valid<SelectionSet<Value>, String> {
        // this field belongs to container_type, so we if want to get this field
        let type_def = match self.0.get_object_type_definition(container_type) {
            Some(type_def) => type_def,
            None => {
                return Valid::fail(format!(
                    "type definition not found for type '{}' ",
                    container_type
                ))
            }
        };

        let mut enriched_selection_set: SelectionSet<Value> = SelectionSet::default();

        for field in selection.into_vec().into_iter() {
            let mut enriched_field = field.clone();
            let field_def = match self.0.get_field(container_type, &field.name) {
                Some(QueryField::Field((def, _))) => def,
                _ => {
                    return Valid::fail(format!(
                        "field definition not found for field '{}' in type '{}' ",
                        field.name, container_type
                    ))
                }
            };

            let enriched_field = if field_def.join_fields.is_empty() {
                // if field doesn't have @join__field directive, then
                // we need to figure out from where this field can be queried

                // 1. this field can be queried form the @join__type -> wherein the key is same as this field.
                // 2. this field can be queried from the @join__type directive's graph where key is none.
                enriched_field
                    .graph
                    .extend(type_def.join_types.iter().filter_map(|jt| {
                        if jt.key.is_none() || jt.key.as_ref().map_or(false, |k| k == &field.name) {
                            Some(jt.graph.clone())
                        } else {
                            None
                        }
                    }));

                enriched_field
            } else {
                enriched_field.join_field(field_def.join_fields.clone())
            };

            let type_name = field_def.of_type.as_type_str();
            if !field.selections.is_empty() {
                self.enrich_information(field.selections, &type_name)
                    .and_then(|enriched_nested_selected_set| {
                        let enriched_field =
                            enriched_field.selections(enriched_nested_selected_set);
                        enriched_selection_set.push(enriched_field);

                        Valid::succeed(())
                    });
            } else {
                enriched_selection_set.push(enriched_field);
            }
        }

        Valid::succeed(enriched_selection_set)
    }
}

impl Transform for Enrich {
    type Value = SelectionSet<Value>;
    type Error = String;

    fn transform(&self, value: Self::Value) -> valid::Valid<Self::Value, Self::Error> {
        // TODO: fix the container type to be dynamic
        self.enrich_information(value, "Query")
    }
}

#[cfg(test)]
mod test {
    use blueprint::Blueprint;

    use super::*;

    fn setup(graphql: &str) -> Index {
        let document = async_graphql_parser::parse_schema(graphql).unwrap();
        let index = Blueprint::parse_doc(document).to_index();
        index
    }

    #[test]
    fn test_enricher_supergraph_1() {
        let query = "query { topProducts { productName: name reviews { body } reviews { id } } }";
        let index = setup(include_str!("../../../blueprint/src/fixtures/router.graphql"));
        let doc = async_graphql_parser::parse_query(query).unwrap();

        // pick the very first operation.
        let op = doc
            .operations
            .iter()
            .next()
            .unwrap()
            .1
            .node
            .selection_set
            .node
            .clone();

        let selection_set: SelectionSet<async_graphql_value::Value> =
            super::SelectionSet::from(&op);

        let enriched_selection_set = Enrich::new(index)
            .transform(selection_set)
            .to_result()
            .unwrap();

        insta::assert_debug_snapshot!(enriched_selection_set)
    }
}
