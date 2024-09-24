use std::collections::{BTreeMap, BTreeSet};

use async_graphql_parser::types::ServiceDocument;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use valid::Valid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blueprint {
    pub definitions: Vec<Definition>,
    pub schema: SchemaDefinition,
    pub directives: Vec<DirectiveDefinition>,
    pub graphs: Vec<JoinGraph>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct GraphId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct JoinGraph {
    pub name: GraphId,
    pub url: url::Url,
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
    pub join_type: Vec<JoinType>,
    pub join_implements: Vec<JoinImplements>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub description: Option<String>,
    pub implements: BTreeSet<String>,
    pub join_type: Vec<JoinType>,
    pub join_implements: Vec<JoinImplements>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<InputFieldDefinition>,
    pub description: Option<String>,
    pub join_type: Vec<JoinType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub enum_values: Vec<EnumValueDefinition>,
    pub join_type: Vec<JoinType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumValueDefinition {
    pub description: Option<String>,
    pub name: String,
    pub directives: Vec<Directive>,
    pub join_enum: Vec<GraphId>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub query: Option<String>,
    pub mutation: Option<String>,
    pub subscription: Option<String>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputFieldDefinition {
    pub name: String,
    pub of_type: Type,
    pub default_value: Option<Value>,
    pub description: Option<String>,
    pub join_field: Vec<JoinField>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub args: Vec<InputFieldDefinition>,
    pub of_type: Type,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub join_field: Vec<JoinField>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Directive {
    pub name: String,
    pub arguments: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectiveDefinition {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<InputFieldDefinition>,
    pub repeatable: bool,
    pub locations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScalarTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub join_type: Vec<JoinType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnionTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub types: BTreeSet<String>,
    pub join_type: Vec<JoinType>,
    pub join_union: Vec<JoinUnion>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinType {
    pub graph: GraphId,
    pub key: Option<String>,
    pub extension: bool,
    pub resolvable: bool,
    pub is_interface_object: bool,
}

impl JoinType {
    pub fn new(graph: String) -> Self {
        Self {
            graph: GraphId(graph),
            key: None,
            extension: false,
            resolvable: true,
            is_interface_object: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinField {
    pub graph: Option<GraphId>,
    pub requires: Option<String>,
    pub provides: Option<String>,
    pub r#type: Option<String>,
    pub external: Option<bool>,
    pub r#override: Option<String>,
    pub used_overridden: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinImplements {
    pub graph: GraphId,
    pub interface: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinUnion {
    pub graph: GraphId,
    pub member: String
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
