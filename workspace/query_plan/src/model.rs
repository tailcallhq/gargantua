use std::ops::{Deref, DerefMut};

use async_graphql::Positioned;
use async_graphql_parser::types::{self as Q};
use blueprint::{Graph, JoinField};
use derive_setters::Setters;

use crate::error::Error;

#[derive(Debug, Clone)]
pub enum QueryPlan<Value> {
    Parallel(Vec<QueryPlan<Value>>),
    Sequence(Vec<QueryPlan<Value>>),
    Fetch {
        service: Graph,
        query: QueryOperation<Value>,
        representations: Option<SelectionSet<Value>>,
        type_name: TypeName,
    },
    Flatten {
        select: Lens,
        plan: Box<QueryPlan<Value>>,
    },
}

#[derive(Debug, Clone)]
pub struct QueryOperation<Value> {
    // TODO: add directives, variables etc.
    pub selection_set: SelectionSet<Value>,
}

impl<A> QueryPlan<A> {
    pub fn fetch(service: Graph, type_name: TypeName, query: SelectionSet<A>) -> Self {
        QueryPlan::Fetch {
            service,
            query: QueryOperation { selection_set: query },
            representations: None,
            type_name,
        }
    }
}

impl QueryPlan<async_graphql_value::Value> {
    // Tries to create a new Query Plan from a GraphQL query and a Blueprint Index.
    pub fn try_new(query: &str) -> Result<Self, Error> {
        let doc = async_graphql_parser::parse_query(query)?;
        let mut parallel = Vec::new();

        // TODO: handle fragments
        // TODO: use named operations
        for (_, op) in doc.operations.iter() {
            let selection = SelectionSet::from(&op.node.selection_set.node);
            let type_name = TypeName::new("Query");
            let service = Graph::new("WIP");
            parallel.push(QueryPlan::fetch(service, type_name, selection));
        }
        Ok(QueryPlan::Parallel(parallel))
    }

    // Sequentially executes one plan after the other
    pub fn and_then(self, select: Lens, plan: QueryPlan<async_graphql_value::Value>) -> Self {
        QueryPlan::Sequence(vec![
            self,
            QueryPlan::Flatten { select, plan: Box::new(plan) },
        ])
    }
}

#[derive(Debug, Clone)]
pub struct TypeName(String);

impl TypeName {
    pub fn new(name: &str) -> Self {
        TypeName(name.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Default, Debug, Clone)]
pub struct SelectionSet<Value>(Vec<Field<Value>>);

impl<A> Deref for SelectionSet<A> {
    type Target = Vec<Field<A>>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<Value> SelectionSet<Value> {
    pub fn new(fields: Vec<Field<Value>>) -> Self {
        Self(fields)
    }

    pub fn push(&mut self, field: Field<Value>) {
        self.0.push(field);
    }

    pub fn into_vec(self) -> Vec<Field<Value>> {
        self.0
    }
}

#[derive(Debug, Clone, Setters)]
pub struct Field<Value> {
    pub name: String,
    pub selections: SelectionSet<Value>,
    pub arguments: Vec<Argument<Value>>,
    pub directives: Vec<Directive<Value>>,

    /// When set to true the field is considered to be used internally for
    /// querying sub-graphs and should not be exposed to the user.
    pub is_hidden: bool,

    /// Possible Graphs from where the field can be queried from.
    pub graph: Vec<Graph>,

    /// Internal readonly information from the Blueprint Index.
    // #[setters(skip)]
    join_field: Vec<JoinField>,
}

impl<A> Field<A> {
    // pub join_field(&mut)

    pub fn new(name: String, selections: SelectionSet<A>) -> Self {
        Field {
            name: name.to_string(),
            selections,
            arguments: Vec::new(),
            directives: Vec::new(),
            is_hidden: false,
            graph: Vec::new(),
            join_field: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Setters)]
pub struct Argument<Value> {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, Setters)]
pub struct Directive<Value> {
    pub name: String,
    pub arguments: Vec<Argument<Value>>,
}

#[derive(Debug, Clone)]
pub enum Lens {
    Field(String),
    Index(usize),
    Combine(Box<Lens>, Box<Lens>),
    ForEach(Box<Lens>),
    Empty,
}

impl Lens {
    pub fn get(&self, _value: serde_json::Value) -> serde_json::Value {
        // TODO: implement
        todo!()
    }
    pub fn set(
        &self,
        _value: serde_json::Value,
        _other_value: serde_json::Value,
    ) -> serde_json::Value {
        // TODO: implement
        todo!()
    }
}

// Correctly implement and add tests
impl From<&Q::SelectionSet> for SelectionSet<async_graphql_value::Value> {
    fn from(node: &Q::SelectionSet) -> SelectionSet<async_graphql_value::Value> {
        let mut selection_set = Vec::new();
        for selection in node.items.iter() {
            let inner_selection = &selection.node;
            match inner_selection {
                Q::Selection::Field(Positioned { node, .. }) => {
                    let field_name = node.name.node.as_str().to_string();
                    let field =
                        Field::new(field_name, SelectionSet::from(&node.selection_set.node));
                    selection_set.push(field);
                }
                Q::Selection::InlineFragment(_) => {
                    todo!()
                }
                Q::Selection::FragmentSpread(_) => {
                    todo!()
                }
            }
        }
        SelectionSet(selection_set)
    }
}

#[cfg(test)]
mod test {
    use insta::assert_debug_snapshot;

    use crate::QueryPlan;

    #[test]
    fn test() {
        let query = "query { topProducts { name reviews { score } reviews { description } } }";
        let actual: QueryPlan<_> = QueryPlan::try_new(query).unwrap();
        assert_debug_snapshot!(actual);
    }

    #[test]
    fn test_complex() {
        let query = r#"
            query getData(
                $userId: String!
                $sortOrder: String = DESC
                $region: String = "EU"
            ) @onQuery {
                me: user(id: $userId) @onField {
                    id
                    username
                    role {
                    id
                    name
                    }
                }
                stores(first: 10, order: $sortOrder, region: $region) {
                    id @onField(data: 1)
                    name @onField(data: { foo: "bar" })
                }
            }
        "#;
        let actual: QueryPlan<_> = QueryPlan::try_new(query).unwrap();
        assert_debug_snapshot!(actual);
    }
}
