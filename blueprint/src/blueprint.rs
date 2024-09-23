use std::collections::{BTreeMap, BTreeSet};

use async_graphql_parser::types::ServiceDocument;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use valid::Valid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blueprint {
    pub definitions: Vec<Definition>,
    pub schema: SchemaDefinition,
}

impl Blueprint {
    pub fn parse(doc: ServiceDocument) -> Valid<Blueprint, String> {
        super::parse::parse(doc)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Definition {
    Interface(InterfaceTypeDefinition),
    Object(ObjectTypeDefinition),
    InputObject(InputObjectTypeDefinition),
    Scalar(ScalarTypeDefinition),
    Enum(EnumTypeDefinition),
    Union(UnionTypeDefinition),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterfaceTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub description: Option<String>,
    pub implements: BTreeSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<InputFieldDefinition>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub enum_values: Vec<EnumValueDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumValueDefinition {
    pub description: Option<String>,
    pub name: String,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub query: String,
    pub mutation: Option<String>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputFieldDefinition {
    pub name: String,
    pub of_type: Type,
    pub default_value: Option<Value>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub args: Vec<InputFieldDefinition>,
    pub of_type: Type,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Directive {
    pub name: String,
    pub arguments: BTreeMap<String, Value>,
    pub index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScalarTypeDefinition {
    pub name: String,
    pub directive: Vec<Directive>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnionTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub types: BTreeSet<String>,
}

/// Type to represent GraphQL type usage with modifiers
/// [spec](https://spec.graphql.org/October2021/#sec-Wrapping-Types)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Type {
    Named {
        /// Name of the type
        name: String,
        /// Flag to indicate the type is required.
        required: bool,
    },
    List {
        /// Type is a list
        of_type: Box<Type>,
        /// Flag to indicate the type is required.
        non_null: bool,
    },
}
