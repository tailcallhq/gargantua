use blueprint::GraphId;
use derive_setters::Setters;

#[derive(Debug, Clone)]
pub enum QueryPlan<Value> {
    Parallel(Vec<QueryPlan<Value>>),
    Sequence(Vec<QueryPlan<Value>>),
    Fetch {
        service: GraphId,
        query: SelectionSet<Value>,
        representations: Option<SelectionSet<Value>>,
        type_name: String,
    },
    Flatten {
        path: Lens,
        plan: Box<QueryPlan<Value>>,
    },
}

#[derive(Debug, Clone, Setters)]
pub struct SelectionSet<Value> {
    fields: Vec<Field<Value>>,
}

#[derive(Debug, Clone, Setters)]
pub struct Field<Value> {
    name: String,
    selections: SelectionSet<Value>,
    arguments: Vec<Argument<Value>>,
    directives: Vec<Directive<Value>>,
}

#[derive(Debug, Clone, Setters)]
pub struct Argument<Value> {
    name: String,
    value: Value,
}

#[derive(Debug, Clone, Setters)]
pub struct Directive<Value> {
    name: String,
    arguments: Vec<Argument<Value>>,
}

#[derive(Debug, Clone)]
pub enum Lens {
    Field(String),
    Index(usize),
    Combine(Box<Lens>, Box<Lens>),
    ForEach(Box<Lens>),
    Empty,
}
