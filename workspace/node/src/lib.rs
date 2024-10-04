use std::collections::HashSet;

pub struct TraitSet<A>(HashSet<A>);

pub struct Node<A, T> {
    pub data: A,
    pub children: Vec<Node<A, T>>,
    pub traits: TraitSet<T>,
}
