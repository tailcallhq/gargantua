use std::fmt::{Debug};

use async_graphql_parser::types::{self as Q};

use crate::error::Error;

#[derive(Debug, Clone)]
pub enum TraitSet<A> {
    Any(Vec<A>),
    Only(A),
    All(Vec<A>),
}

impl<A> Default for TraitSet<A> {
    fn default() -> Self {
        TraitSet::Any(Vec::new())
    }
}

impl<A: Clone + Eq + std::hash::Hash> From<&[A]> for TraitSet<A> {
    fn from(value: &[A]) -> Self {
        TraitSet::Any(value.to_vec())
    }
}

#[derive(Debug, Clone)]
pub struct Node<A, T> {
    pub traits: TraitSet<T>,
    pub data: A,
    pub children: Vec<Node<A, T>>,
}

impl<A, T> Node<A, T> {
    pub fn try_from(sel: Q::SelectionSet) -> Result<Self, Error> {
        todo!()
    }
}

fn cartesian_product<T: Clone>(input: &[Vec<T>]) -> Vec<Vec<T>> {
    let mut result = vec![vec![]];
    for pool in input {
        let mut temp = Vec::new();
        for partial in &result {
            for item in pool {
                let mut new_partial = partial.clone();
                new_partial.push(item.clone());
                temp.push(new_partial);
            }
        }
        result = temp;
    }
    result
}

impl<A: Clone, T: Clone> Node<A, T> {
    pub fn generate_options(&self) -> Vec<Self> {
        match &self.traits {
            TraitSet::Any(set) => {
                let mut out: Vec<Self> = Vec::new();
                let children_set = cartesian_product(
                    &self
                        .children
                        .iter()
                        .map(|c| c.generate_options())
                        .collect::<Vec<_>>(),
                );

                for children in children_set {
                    for tr in set.iter() {
                        out.push(Node {
                            data: self.data.clone(),
                            children: children.clone(),
                            traits: TraitSet::Only(tr.clone()),
                        });
                    }

                    if set.len() > 1 {
                        out.push(Node {
                            data: self.data.clone(),
                            children: children.clone(),
                            traits: TraitSet::All(set.clone()),
                        });
                    }
                }

                out
            }
            _ => vec![self.clone()],
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_options() {
        // Create traits
        let location = "Location";
        let reviews = "Reviews";

        // Create parent node
        let parent_node = Node {
            data: "test",
            children: vec![
                Node {
                    data: "p",
                    children: vec![],
                    traits: TraitSet::from(&[reviews][..]),
                },
                Node {
                    data: "r",
                    children: vec![],
                    traits: TraitSet::from(&[location, reviews][..]),
                },
            ],
            traits: TraitSet::from(&[location, reviews][..]),
        };

        // Generate options
        let options = parent_node.generate_options();
        assert_eq!(options.len(), 9);
        insta::assert_debug_snapshot!(options);
    }
}
