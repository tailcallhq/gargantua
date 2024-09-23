use std::fmt::{Debug, Formatter};

use blueprint::Type;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OperationPlan<Input> {
    pub operation_type: OperationType,
    pub nested: Vec<Field<Input>>,
    pub is_introspection_query: bool,
}

/// The type of an operation; `query`, `mutation` or `subscription`.
///
/// [Reference](https://spec.graphql.org/October2021/#OperationType).
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// A query.
    Query,
    /// A mutation.
    Mutation,
    /// A subscription.
    Subscription,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FieldId(usize);

#[derive(Clone, Debug, PartialEq)]
pub struct Variable(String);

impl Variable {
    pub fn new(name: String) -> Self {
        Variable(name)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Clone)]
pub struct ArgId(usize);

impl Debug for ArgId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ArgId {
    pub fn new(id: usize) -> Self {
        ArgId(id)
    }
}

#[derive(Debug, Clone)]
pub struct Arg<Input> {
    pub id: ArgId,
    pub name: String,
    pub type_of: Type,
    pub value: Option<Input>,
    pub default_value: Option<Input>,
}

#[derive(Clone)]
pub struct Field<Input> {
    pub id: FieldId,
    /// Name of key in the value object for this field
    pub name: String,
    /// Output name (i.e. with alias) that should be used for the result value
    /// of this field
    pub output_name: String,
    pub type_of: Type,
    /// Specifies the name of type used in condition to fetch that field
    /// The type could be anything from graphql type system:
    /// interface, type, union, input type.
    /// See [spec](https://spec.graphql.org/October2021/#sec-Type-Conditions)
    pub type_condition: Option<String>,
    pub skip: Option<Variable>,
    pub include: Option<Variable>,
    pub args: Vec<Arg<Input>>,
    pub children: Vec<Field<Input>>,
    pub pos: Pos,
    pub directives: Vec<Directive<Input>>,
}

#[derive(Clone, Debug)]
pub struct Directive<Input> {
    pub name: String,
    pub arguments: Vec<(String, Input)>,
}

impl<Input> Directive<Input> {
    pub fn try_map<Output, Error>(
        self,
        map: impl Fn(Input) -> Result<Output, Error>,
    ) -> Result<Directive<Output>, Error> {
        Ok(Directive {
            name: self.name,
            arguments: self
                .arguments
                .into_iter()
                .map(|(k, v)| map(v).map(|mapped_value| (k, mapped_value)))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Original position of an element in source code.
///
/// You can serialize and deserialize it to the GraphQL `locations` format
/// ([reference](https://spec.graphql.org/October2021/#sec-Errors)).
#[derive(
    Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Default, Hash, Serialize, Deserialize,
)]
pub struct Pos {
    /// One-based line number.
    pub line: usize,

    /// One-based column number.
    pub column: usize,
}
