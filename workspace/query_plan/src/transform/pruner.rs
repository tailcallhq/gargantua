use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use valid::{Transform, Valid, Validator};

use crate::{Field, SelectionSet};

// prunes out possible subgraphs from node.
struct Pruner<T>(PhantomData<T>);

impl<T> Pruner<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Clone + Default> Transform for Pruner<T> {
    type Value = Field<T>;
    type Error = String;
    fn transform(&self, input: Self::Value) -> Valid<Self::Value, Self::Error> {
        let subgraphs = Self::collect_subgraph(&input);
        let field_to_subgraphs = Self::collect_fields(&input);
        Self::minimum_set_cover(subgraphs, field_to_subgraphs).and_then(|required_subgraphs| {
            let output = Self::prune_field(input, &required_subgraphs, None);
            Valid::succeed(output)
        })
    }
}

impl<T: Clone + Default> Pruner<T> {
    /// Prunes a field and its subfields in a two-step process:
    /// 1. Removes subgraphs not present in the `required_subgraphs` set.
    /// 2. If there's a common subgraph with the parent, further prunes based on `parent_subgraphs`.
    ///
    /// This function recursively applies the pruning process to all nested fields.
    fn prune_field(
        field: Field<T>,
        required_subgraphs: &HashSet<String>,
        parent_subgraphs: Option<&HashSet<String>>,
    ) -> Field<T> {
        let field_subgraphs: HashSet<String> = field
            .join_field
            .iter()
            .filter_map(|sub| sub.graph.as_ref().map(|g| g.as_str().to_string()))
            .collect();

        let is_common = parent_subgraphs
            .map(|ps| !field_subgraphs.is_disjoint(ps))
            .unwrap_or(false);

        let pruned_join_field = field
            .join_field
            .into_iter()
            .filter(|sub| {
                sub.graph.as_ref().map_or(true, |g| {
                    let contains = required_subgraphs.contains(g.as_str());
                    if is_common {
                        contains && parent_subgraphs.map_or(true, |ps| ps.contains(g.as_str()))
                    } else {
                        contains
                    }
                })
            })
            .collect();

        let pruned_fields = field
            .selections
            .iter()
            .map(|child| {
                Self::prune_field(child.to_owned(), required_subgraphs, Some(&field_subgraphs))
            })
            .collect::<Vec<Field<T>>>();

        Field {
            name: field.name,
            join_field: pruned_join_field,
            selections: SelectionSet::new(pruned_fields),
            ..field
        }
    }

    // collects the all subgraphs into hashset.
    fn collect_subgraph(field: &Field<T>) -> HashSet<String> {
        let mut subgraphs = HashSet::new();
        for sub in field.join_field.iter() {
            if let Some(g) = sub.graph.as_ref() {
                subgraphs.insert(g.as_str().to_string());
            }
        }

        for child in field.selections.iter() {
            let child_sub = Self::collect_subgraph(child);
            subgraphs.extend(child_sub);
        }

        subgraphs
    }

    // collects the fields and maps like field_name: [subgraphs]
    // TODO: refactor the HashMap to hold field_id as key instead of field name
    fn collect_fields(field: &Field<T>) -> HashMap<String, HashSet<String>> {
        let mut field_to_subgraph: HashMap<String, HashSet<String>> = HashMap::new();
        for sub in field.join_field.iter() {
            if let Some(g) = sub.graph.as_ref() {
                field_to_subgraph
                    .entry(field.name.to_string())
                    .or_default()
                    .insert(g.as_str().to_string());
            }
        }

        for child in field.selections.iter() {
            let child_sub = Self::collect_fields(child);
            field_to_subgraph.extend(child_sub);
        }

        field_to_subgraph
    }

    fn minimum_set_cover(
        subgraphs: HashSet<String>,
        field_to_subgraphs: HashMap<String, HashSet<String>>,
    ) -> Valid<HashSet<String>, String> {
        let mut min_cover = HashSet::new();
        let mut uncovered_fields: HashSet<String> = field_to_subgraphs.keys().cloned().collect();

        while !uncovered_fields.is_empty() {
            let mut best_subgraph = None;
            let mut max_coverage = 0;

            for subgraph in subgraphs.iter() {
                let coverage = field_to_subgraphs
                    .iter()
                    .filter(|(field, sgs)| {
                        uncovered_fields.contains(*field) && sgs.contains(subgraph)
                    })
                    .count();

                if coverage > max_coverage {
                    max_coverage = coverage;
                    best_subgraph = Some(subgraph);
                }
            }

            if let Some(subgraph) = best_subgraph {
                min_cover.insert(subgraph.clone());
                uncovered_fields.retain(|field| !field_to_subgraphs[field].contains(subgraph));
            } else {
                return Valid::fail("Invalid Input: Failed to find a valid set cover".to_string());
            }
        }

        Valid::succeed(min_cover)
    }
}

#[cfg(test)]
mod test {
    use blueprint::{Graph, JoinField, JoinFieldParsed};
    use valid::{Transform, Validator};

    use crate::{Field, SelectionSet};

    use super::Pruner;

    /// topProducts {   [Product]
    ///     name        [Product]
    ///     reviews {   [Reviews]
    ///         body    [Reviews, Unknown]
    ///     }
    /// }
    /// with set cover we can figure out that all fields in graph can be easily resolved by
    /// `Product` and `Reviews` subgraphs only. so we can easily prune out the `Unknow` subgraph.
    ///
    /// We need another step of pruning.
    ///
    /// set cover pruns out subgraphs by looking at global picture but there's one more step that we can do
    /// reduce the subgraphs, i mean we can prune based on parent's subgraphs.
    /// eg. reviews [Reviews]
    ///         body    [Reviews, Unknown]  -> so we can remove the `Unknown`
    ///         test    [Something]         -> there's no common so keep the `Something` as is.
    ///
    ///
    /// eg.
    /// topProducts {   [Products]
    ///     name    [Products]
    ///     reviews {   [Reviews]
    ///         body    [Reviews, Unknown]
    ///     }
    ///     test    [Unknown]
    /// }
    #[test]
    fn test() {
        let reviews_subgraph = Graph::new("Reviews");
        let product_subgraph = Graph::new("Product");
        let unknown_subgraph = Graph::new("Unknown");
        let something_subgraph = Graph::new("Something");

        let name: Field<String> =
            Field::new("name".into(), SelectionSet::default()).join_field(vec![
                JoinFieldParsed::from(JoinField::new(product_subgraph.clone())),
            ]);
        let test: Field<String> =
            Field::new("test".into(), SelectionSet::default()).join_field(vec![
                JoinFieldParsed::from(JoinField::new(something_subgraph.clone())),
            ]);
        let body: Field<String> =
            Field::new("body".into(), SelectionSet::default()).join_field(vec![
                JoinFieldParsed::from(JoinField::new(reviews_subgraph.clone())),
                JoinFieldParsed::from(JoinField::new(unknown_subgraph.clone())),
            ]);
        let reviews = Field::new("reviews".into(), SelectionSet::new(vec![body])).join_field(vec![
            JoinFieldParsed::from(JoinField::new(reviews_subgraph.clone())),
        ]);
        let base_field = Field::new(
            "topProducts".into(),
            SelectionSet::new(vec![name, reviews, test]),
        )
        .join_field(vec![JoinFieldParsed::from(JoinField::new(
            product_subgraph.clone(),
        ))]);

        let pruned_selection_set = Pruner::new().transform(base_field).to_result().unwrap();
        insta::assert_debug_snapshot!(pruned_selection_set);
    }
}
