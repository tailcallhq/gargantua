use std::collections::HashSet;

use async_graphql_parser::types::{self as Q};

use crate::error::Error;

pub struct TraitSet<A>(HashSet<A>);

pub struct Node<A, T, P> {
    pub data: A,
    pub children: Vec<Node<A, T, P>>,
    pub traits: TraitSet<T>,
    pub primary: P,
}

impl<A, T> Node<A, T, ()> {
    pub fn try_from(sel: Q::SelectionSet) -> Result<Self, Error> {
        todo!()
    }
}
