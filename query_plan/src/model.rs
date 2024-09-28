use blueprint::{Graph, Index};
use derive_setters::Setters;
use valid::Validator;

use crate::error::Error;
use crate::Builder;

#[derive(Debug, Clone)]
pub enum QueryPlan<Value> {
    Parallel(Vec<QueryPlan<Value>>),
    Sequence(Vec<QueryPlan<Value>>),
    Fetch {
        service: Graph,
        query: SelectionSet<Value>,
        representations: Option<SelectionSet<Value>>,
        type_name: TypeName,
    },
    Flatten {
        path: Lens,
        plan: Box<QueryPlan<Value>>,
    },
}

impl<A> QueryPlan<A> {
    pub fn fetch(service: Graph, type_name: TypeName, query: SelectionSet<A>) -> Self {
        QueryPlan::Fetch { service, query, representations: None, type_name }
    }

    pub fn try_new(query: String, index: Index) -> Result<Self, Error> {
        let doc = async_graphql_parser::parse_query(&query)?;
        let builder: Builder<A> = Builder::<A>::new(index);
        Ok(builder.build(&doc).to_result()?)
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

#[derive(Default, Debug, Clone, Setters)]
pub struct SelectionSet<Value> {
    pub fields: Vec<Field<Value>>,
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
