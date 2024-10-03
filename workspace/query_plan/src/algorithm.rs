use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::{Field, SelectionSet};

fn set_cover(fields: Vec<String>, subsets: HashMap<String, Vec<String>>) -> HashSet<String> {
    let mut covered = HashSet::new();
    let mut selected_subsets = HashSet::new();

    while covered.len() < fields.len() {
        let mut best_subset = None;
        let mut best_covered = 0;

        for (subset_name, elements) in &subsets {
            // Count how many new fields this subset covers
            let newly_covered: usize = elements.iter().filter(|e| !covered.contains(e)).count();

            if newly_covered > best_covered {
                best_covered = newly_covered;
                best_subset = Some(subset_name);
            }
        }

        // If no subset can cover new fields, we are done
        if let Some(subset) = best_subset {
            selected_subsets.insert(subset.to_string());
            if let Some(elements) = subsets.get(subset) {
                for e in elements {
                    covered.insert(e);
                }
            }
        } else {
            break; // No more subsets to cover new fields
        }
    }

    selected_subsets
}

pub fn alogrithm<T: Debug + Clone + Default>(field: &Field<T>) -> &Field<T> {
    if field.selections.is_empty() {
        return field;
    }

    let mut local_paths = vec![];
    let mut map: HashMap<String, Vec<String>> = HashMap::default();
    let mut field_names = Vec::new();
    for sub_selection in field.selections.iter() {
        let field = alogrithm(sub_selection);
        field_names.push(sub_selection.name.to_string());
        for subgraph in field.join_field.iter() {
            let name = subgraph.clone().graph.unwrap().as_str().to_string();
            if let Some(inner_fields) = map.get_mut(&name) {
                inner_fields.push(sub_selection.name.to_string());
            } else {
                map.insert(name, vec![sub_selection.name.to_string()]);
            }
        }
        local_paths.push(field);
    }

    let required_maps = set_cover(field_names, map.clone());
    if required_maps.len() > 1 {
        println!("[Federated Calls] => all mappings {:#?}", map);
        println!(
            "[Federated Calls] => we can get all fields with {:#?}",
            required_maps
        );

        let _ = merge(
            field,
            &local_paths.into_iter().cloned().collect(),
            &required_maps,
        );

        // probably federated calls.
    } else {
        let subgraph = required_maps.iter().next().unwrap();
        println!(
            " {:#?} => {:#?} ",
            required_maps,
            map.get(subgraph).unwrap()
        );
    }
    field
}

fn merge<T: Debug + Clone + Default>(
    respect_to: &Field<T>,
    needs_merging: &Vec<Field<T>>,
    allowed_graphs: &HashSet<String>,
) -> () {
    let mut belongs_to = vec![]; // this needs to packed as one.
                                 // whatever present in p -> needs to be sequential(may be).
    let mut p = vec![];
    for field in needs_merging {
        let belongs = field.graph.iter().any(|g| respect_to.graph.contains(g));

        if belongs {
            belongs_to.push(field.clone());
        } else {
            belongs_to.push(Field::new("__typename".into(), SelectionSet::default()));
            belongs_to.push(Field::new("id".into(), SelectionSet::default())); // TODO: fix this with correct ID.

            // this federated call -> reason being this belongs to different subgraph than
            // it's parent.
            let mut cloned_f = field.clone();
            cloned_f.join_field.retain(|jf| {
                if let Some(g) = &jf.graph {
                    allowed_graphs.contains(g.as_str())
                } else {
                    false
                }
            });
            p.push(cloned_f);
        }
    }

    let mut groups = vec![];

    for single_p in p.iter() {
        groups.push(merge(single_p, &p, allowed_graphs));
    }
}

#[cfg(test)]
mod test {
    use blueprint::{Graph, JoinFieldParsed};

    use super::*;
    use crate::{Argument, Directive, Field, SelectionSet};

    fn build_location_field() -> Field<async_graphql_value::Value> {
        Field {
            name: "location".to_string(),
            alias: None,
            selections: SelectionSet::new(vec![
                Field::new("n1".to_string(), SelectionSet::new(vec![])).join_field(vec![
                    JoinFieldParsed::new(Graph::new("Location"), None, None),
                ]),
                Field::new("n2".to_string(), SelectionSet::new(vec![])).join_field(vec![
                    JoinFieldParsed::new(Graph::new("Reviews"), None, None),
                ]),
                Field::new("n3".to_string(), SelectionSet::new(vec![])).join_field(vec![
                    JoinFieldParsed::new(Graph::new("Location"), None, None),
                ]),
                Field::new("n4".to_string(), SelectionSet::new(vec![])).join_field(vec![
                    JoinFieldParsed::new(Graph::new("Location"), None, None),
                    JoinFieldParsed::new(Graph::new("Auth"), None, None),
                ]),
            ]),
            arguments: Vec::new(),
            directives: Vec::new(),
            is_hidden: false,
            graph: vec![Graph::new("Location")],
            join_field: vec![JoinFieldParsed::new(Graph::new("Location"), None, None)],
            field_type: None,
            parent_type: None,
        }
    }

    #[test]
    fn test() {
        let field = build_location_field();
        let _ = alogrithm(&field);
    }

    #[test]
    fn test_complex() {
        let field: Field<async_graphql_value::Value> = Field {
            name: "fizz".to_string(),
            alias: Some("buzz".to_string()),
            selections: SelectionSet::new(vec![
                Field::new("id".to_string(), SelectionSet::new(vec![])),
                Field::new(
                    "field_1".to_string(),
                    SelectionSet::new(vec![
                        Field::new("field_1_1".to_string(), SelectionSet::new(vec![])),
                        Field::new("field_1_2".to_string(), SelectionSet::new(vec![])).join_field(
                            vec![
                                JoinFieldParsed::new(Graph::new("T2"), None, None),
                                JoinFieldParsed::new(Graph::new("T3"), None, None),
                            ],
                        ),
                    ]),
                ),
                Field::new(
                    "field_2".to_string(),
                    SelectionSet::new(vec![
                        Field::new("field_2_1".to_string(), SelectionSet::new(vec![])),
                        Field::new("field_2_2".to_string(), SelectionSet::new(vec![]))
                            .join_field(vec![JoinFieldParsed::new(Graph::new("T2"), None, None)]),
                        Field::new(
                            "field_2_3".to_string(),
                            SelectionSet::new(vec![Field::new(
                                "field_2_1".to_string(),
                                SelectionSet::new(vec![]),
                            )]),
                        )
                        .join_field(vec![JoinFieldParsed::new(
                            Graph::new("T1"),
                            None,
                            None,
                        )]),
                    ]),
                )
                .join_field(vec![
                    JoinFieldParsed::new(Graph::new("T1"), None, None),
                    JoinFieldParsed::new(Graph::new("T2"), None, None),
                ]),
                Field::new(
                    "field_3".to_string(),
                    SelectionSet::new(vec![Field::new(
                        "field_3_1".to_string(),
                        SelectionSet::new(vec![]),
                    )
                    .join_field(vec![JoinFieldParsed::new(Graph::new("T4"), None, None)])]),
                )
                .join_field(vec![
                    JoinFieldParsed::new(Graph::new("T4"), None, None),
                    JoinFieldParsed::new(Graph::new("T5"), None, None),
                ]),
            ]),
            arguments: vec![Argument {
                name: "fooArg".to_string(),
                value: async_graphql_value::Value::from_json(
                    serde_json::json!({ "fizz": false, "buzz": 12 }),
                )
                .unwrap(),
            }],
            directives: vec![Directive {
                name: "barDirective".to_string(),
                arguments: vec![Argument {
                    name: "test".to_string(),
                    value: async_graphql_value::Value::from_json(
                        serde_json::json!({ "spam": false, "eggs": 12 }),
                    )
                    .unwrap(),
                }],
            }],
            is_hidden: false,
            graph: vec![Graph::new("T1"), Graph::new("T2"), Graph::new("T3")],
            join_field: vec![
                JoinFieldParsed::new(Graph::new("T1"), None, None),
                JoinFieldParsed::new(Graph::new("T2"), None, None),
                JoinFieldParsed::new(Graph::new("T3"), None, None),
            ],
            // TODO: something like that might be useful join_types: vec![(T1, "id"), (T2, "id"), (T3, "id")]
            field_type: None,
            parent_type: None,
        };
        let _ = alogrithm(&field);
    }
}
