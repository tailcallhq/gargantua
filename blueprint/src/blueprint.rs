use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Blueprint {
    pub definitions: Vec<Definition>,
    pub schema: SchemaDefinition,
}

#[derive(Clone, Debug)]
pub enum Definition {
    Interface(InterfaceTypeDefinition),
    Object(ObjectTypeDefinition),
    InputObject(InputObjectTypeDefinition),
    Scalar(ScalarTypeDefinition),
    Enum(EnumTypeDefinition),
    Union(UnionTypeDefinition),
}

#[derive(Clone, Debug)]
pub struct InterfaceTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub description: Option<String>,
    pub implements: BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub struct InputObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<InputFieldDefinition>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EnumTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub enum_values: Vec<EnumValueDefinition>,
}

#[derive(Clone, Debug)]
pub struct EnumValueDefinition {
    pub description: Option<String>,
    pub name: String,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug)]
pub struct SchemaDefinition {
    pub query: String,
    pub mutation: Option<String>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug)]
pub struct InputFieldDefinition {
    pub name: String,
    pub of_type: Type,
    pub default_value: Option<Value>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FieldDefinition {
    pub name: String,
    pub args: Vec<InputFieldDefinition>,
    pub of_type: Type,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct Directive {
    pub name: String,
    pub arguments: BTreeMap<String, Value>,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub struct ScalarTypeDefinition {
    pub name: String,
    pub directive: Vec<Directive>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct UnionTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub types: BTreeSet<String>,
}

/// Type to represent GraphQL type usage with modifiers
/// [spec](https://spec.graphql.org/October2021/#sec-Wrapping-Types)
#[derive(Clone, Debug)]
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
