use std::collections::{BTreeMap, BTreeSet};

use async_graphql_parser::{Pos, Positioned};
use async_graphql_value::{ConstValue, Name};
use valid::{Valid, Validator};

use crate::{
    Blueprint, Definition, Directive, DirectiveDefinition, EnumValueDefinition, FieldDefinition,
    InputFieldDefinition, JoinEnum, JoinField, JoinGraph, JoinImplements, JoinType, JoinUnion,
    SchemaDefinition, Type,
};

macro_rules! extract_join {
    ($item:expr, $name:expr, $ty:ty) => {
        $item
            .filter(|directive| directive.name == $name)
            .map(|directive| {
                let t: $ty = serde_json::from_value(directive.arguments).unwrap();
                t
            })
            .collect::<Vec<$ty>>()
    };
}

// TODO: drop Valid from here
// Reading a super-graph configuration is infallible
pub fn parse(doc: async_graphql_parser::types::ServiceDocument) -> Valid<Blueprint, String> {
    let mut root_schema = Valid::succeed(SchemaDefinition {
        query: None,
        mutation: None,
        subscription: None,
        directives: Vec::new(),
    });
    let mut definitions = BTreeMap::<String, Valid<Definition, String>>::new();
    let mut directives = BTreeMap::<String, Valid<DirectiveDefinition, String>>::new();

    for definition in doc.definitions.into_iter() {
        match definition {
            async_graphql_parser::types::TypeSystemDefinition::Schema(Positioned {
                pos: _,
                node: schema_node,
            }) => {
                let schema = parse_schema(schema_node);

                root_schema = root_schema
                    .zip(schema)
                    .and_then(|(mut root_schema, schema)| {
                        if let (None, Some(query)) = (&root_schema.query, schema.query) {
                            root_schema.query = Some(query)
                        }

                        if let (None, Some(mutation)) = (&root_schema.mutation, schema.mutation) {
                            root_schema.mutation = Some(mutation)
                        }

                        if let (None, Some(subscription)) =
                            (&root_schema.subscription, schema.subscription)
                        {
                            root_schema.subscription = Some(subscription)
                        }

                        // TODO: validate that non-repetitive directive is not defined twice
                        let directives = root_schema
                            .directives
                            .clone()
                            .into_iter()
                            .chain(schema.directives.into_iter())
                            .collect::<Vec<_>>();

                        root_schema.directives = directives;
                        Valid::succeed(root_schema)
                    });
            }
            async_graphql_parser::types::TypeSystemDefinition::Type(Positioned {
                pos: type_pos,
                node: type_node,
            }) => {
                let name = type_node.name.clone().into_inner().to_string();

                // TODO: properly merge types
                let definition = parse_type(type_node);
                match definitions.remove(&name) {
                    Some(res) => {
                        let err = format!(
                            "The type `{}` has been already defined, {:?}",
                            name, type_pos
                        );
                        let definition = match res.to_result() {
                            Ok(_) => Valid::fail(err),
                            Err(e) => {
                                let e = e.append(err);
                                Valid::from_validation_err(e)
                            }
                        };
                        definitions.insert(name, definition);
                    }
                    None => {
                        definitions.insert(name, definition);
                    }
                }
            }
            async_graphql_parser::types::TypeSystemDefinition::Directive(Positioned {
                pos: directive_pos,
                node: directive_node,
            }) => {
                let name = directive_node.name.clone().into_inner().to_string();
                match directives.remove(&name) {
                    Some(existing) => {
                        let error = format!(
                            "The directive `{}` has been defined twice,{:?}",
                            name, directive_pos
                        );
                        match existing.to_result() {
                            Ok(_) => {
                                directives.insert(name, Valid::fail(error));
                            }
                            Err(e) => {
                                let e = e.append(error);
                                directives.insert(name, Valid::from_validation_err(e));
                            }
                        }
                    }
                    None => {
                        let directive = parse_directive_definition(directive_node);
                        directives.insert(name.clone(), directive);
                    }
                };
            }
        }
    }
    let definitions =
        Valid::from_iter(definitions.clone().into_values().collect::<Vec<_>>(), |d| d);
    let directives = Valid::from_iter(directives.into_values().collect::<Vec<_>>(), |d| d);

    let join_graphs = definitions.clone().and_then(|definitions| {
        let join_graph_enum = definitions.clone().into_iter().find_map(|definition| {
            if let Definition::Enum(enumeration) = definition {
                if enumeration.name == "join__Graph" {
                    let data =
                        enumeration
                            .enum_values
                            .into_iter()
                            .fold(Vec::new(), |mut acc, cur| {
                                let mut join_graphs: Vec<JoinGraph> = extract_join!(
                                    cur.directives.into_iter(),
                                    "join__graph",
                                    JoinGraph
                                );
                                acc.append(&mut join_graphs);
                                acc
                            });

                    Some(data)
                } else {
                    None
                }
            } else {
                None
            }
        });
        Valid::from_option(
            join_graph_enum,
            "The `join__Graph` enumeration is missing".into(),
        )
    });

    root_schema
        .fuse(definitions)
        .fuse(directives)
        .fuse(join_graphs)
        .map(|(schema, definitions, directives, join_graphs)| Blueprint {
            definitions,
            schema,
            directives,
            join_graphs,
        })
}

fn parse_directive_definition(
    directive_node: async_graphql_parser::types::DirectiveDefinition,
) -> Valid<DirectiveDefinition, String> {
    let name = directive_node.name.into_inner().to_string();
    let repeatable = directive_node.is_repeatable;
    let description = directive_node.description.map(|d| d.to_string());

    let arguments = Valid::from_iter(
        directive_node.arguments,
        |Positioned { pos: _, node: input_field_node }| parse_input_field(input_field_node),
    );

    let locations: Vec<String> = directive_node
        .locations
        .into_iter()
        .map(|loc| match loc.into_inner() {
            async_graphql_parser::types::DirectiveLocation::Query => "Query".into(),
            async_graphql_parser::types::DirectiveLocation::Mutation => "Mutation".into(),
            async_graphql_parser::types::DirectiveLocation::Subscription => "Subscription".into(),
            async_graphql_parser::types::DirectiveLocation::Field => "Field".into(),
            async_graphql_parser::types::DirectiveLocation::FragmentDefinition => {
                "FragmentDefinition".into()
            }
            async_graphql_parser::types::DirectiveLocation::FragmentSpread => {
                "FragmentSpread".into()
            }
            async_graphql_parser::types::DirectiveLocation::InlineFragment => {
                "InlineFragment".into()
            }
            async_graphql_parser::types::DirectiveLocation::Schema => "Schema".into(),
            async_graphql_parser::types::DirectiveLocation::Scalar => "Scalar".into(),
            async_graphql_parser::types::DirectiveLocation::Object => "Object".into(),
            async_graphql_parser::types::DirectiveLocation::FieldDefinition => {
                "FieldDefinition".into()
            }
            async_graphql_parser::types::DirectiveLocation::ArgumentDefinition => {
                "ArgumentDefinition".into()
            }
            async_graphql_parser::types::DirectiveLocation::Interface => "Interface".into(),
            async_graphql_parser::types::DirectiveLocation::Union => "Union".into(),
            async_graphql_parser::types::DirectiveLocation::Enum => "Enum".into(),
            async_graphql_parser::types::DirectiveLocation::EnumValue => "EnumValue".into(),
            async_graphql_parser::types::DirectiveLocation::InputObject => "InputObject".into(),
            async_graphql_parser::types::DirectiveLocation::InputFieldDefinition => {
                "InputFieldDefinition".into()
            }
            async_graphql_parser::types::DirectiveLocation::VariableDefinition => {
                "VariableDefinition".into()
            }
        })
        .collect();

    arguments.map(|arguments| DirectiveDefinition {
        name,
        description,
        arguments,
        repeatable,
        locations,
    })
}

fn parse_type(type_node: async_graphql_parser::types::TypeDefinition) -> Valid<Definition, String> {
    let name = type_node.name.to_string();
    let description = type_node.description.map(|d| d.to_string());
    let directives = Valid::from_iter(
        type_node.directives.clone(),
        |Positioned { pos: _, node: directive_node }| parse_directive(directive_node),
    );

    match type_node.kind {
        async_graphql_parser::types::TypeKind::Scalar => directives.map(|directives| {
            let join_types: Vec<JoinType> =
                extract_join!(directives.clone().into_iter(), "join__type", JoinType);
            Definition::Scalar(crate::ScalarTypeDefinition {
                name,
                directives,
                description,
                join_types,
            })
        }),
        async_graphql_parser::types::TypeKind::Object(object_type) => {
            let fields = Valid::from_iter(
                object_type.fields,
                |Positioned { pos: _, node: field_node }| parse_field(field_node),
            );

            let implements = object_type
                .implements
                .into_iter()
                .map(|name| name.to_string())
                .collect::<BTreeSet<String>>();

            directives.zip(fields).map(|(directives, fields)| {
                let join_types: Vec<JoinType> =
                    extract_join!(directives.clone().into_iter(), "join__type", JoinType);
                let join_implements: Vec<JoinImplements> = extract_join!(
                    directives.clone().into_iter(),
                    "join__implements",
                    JoinImplements
                );

                Definition::Object(crate::ObjectTypeDefinition {
                    name,
                    fields,
                    description,
                    implements,
                    join_types,
                    join_implements,
                })
            })
        }
        async_graphql_parser::types::TypeKind::Interface(interface_type) => {
            let fields = Valid::from_iter(
                interface_type.fields,
                |Positioned { pos: _, node: field_node }| parse_field(field_node),
            );

            directives.zip(fields).map(|(directives, fields)| {
                let join_types: Vec<JoinType> =
                    extract_join!(directives.clone().into_iter(), "join__type", JoinType);
                let join_implements: Vec<JoinImplements> = extract_join!(
                    directives.clone().into_iter(),
                    "join__implements",
                    JoinImplements
                );
                Definition::Interface(crate::InterfaceTypeDefinition {
                    name,
                    fields,
                    description,
                    join_implements,
                    join_types,
                })
            })
        }
        async_graphql_parser::types::TypeKind::Union(union_type) => {
            let types = union_type
                .members
                .into_iter()
                .map(|type_name| type_name.into_inner().to_string())
                .collect();

            directives.map(|directives| {
                let join_types: Vec<JoinType> =
                    extract_join!(directives.clone().into_iter(), "join__type", JoinType);
                let join_unions: Vec<JoinUnion> = extract_join!(
                    directives.clone().into_iter(),
                    "join__unionMember",
                    JoinUnion
                );

                Definition::Union(crate::UnionTypeDefinition {
                    name,
                    directives,
                    description,
                    types,
                    join_types,
                    join_unions,
                })
            })
        }
        async_graphql_parser::types::TypeKind::Enum(enum_type) => {
            let enum_values = Valid::from_iter(
                enum_type.values,
                |Positioned { pos: _, node: enum_node }| parse_enum(enum_node),
            );

            directives
                .zip(enum_values)
                .map(|(directives, enum_values)| {
                    let join_types: Vec<JoinType> =
                        extract_join!(directives.clone().into_iter(), "join__type", JoinType);
                    Definition::Enum(crate::EnumTypeDefinition {
                        name,
                        directives,
                        description,
                        enum_values,
                        join_types,
                    })
                })
        }
        async_graphql_parser::types::TypeKind::InputObject(input_object_type) => {
            let fields = Valid::from_iter(
                input_object_type.fields,
                |Positioned { pos: _, node: input_field_node }| parse_input_field(input_field_node),
            );

            directives.zip(fields).map(|(directives, fields)| {
                let join_types: Vec<JoinType> =
                    extract_join!(directives.into_iter(), "join__type", JoinType);
                Definition::InputObject(crate::InputObjectTypeDefinition {
                    name,
                    fields,
                    description,
                    join_types,
                })
            })
        }
    }
}

fn parse_enum(
    enum_node: async_graphql_parser::types::EnumValueDefinition,
) -> Valid<EnumValueDefinition, String> {
    let name = enum_node.value.to_string();
    let description = enum_node.description.map(|d| d.to_string());
    let directives = Valid::from_iter(
        enum_node.directives,
        |Positioned { pos: _, node: directive_node }| parse_directive(directive_node),
    );

    directives.map(|directives| {
        let join_enums: Vec<JoinEnum> =
            extract_join!(directives.clone().into_iter(), "join__enumValue", JoinEnum);
        EnumValueDefinition { description, name, directives, join_enums }
    })
}

fn parse_field(
    field_node: async_graphql_parser::types::FieldDefinition,
) -> Valid<FieldDefinition, String> {
    let name = field_node.name.to_string();
    let description = field_node.description.map(|d| d.to_string());
    let of_type = map_type(&field_node.ty.into_inner());

    let args = Valid::from_iter(
        field_node.arguments,
        |Positioned { pos: _, node: arg_node }| parse_input_field(arg_node),
    );

    let directives = Valid::from_iter(
        field_node.directives,
        |Positioned { pos: _, node: directive_node }| parse_directive(directive_node),
    );

    args.zip(directives).map(|(args, directives)| {
        let join_fields: Vec<JoinField> =
            extract_join!(directives.clone().into_iter(), "join__field", JoinField);

        FieldDefinition { name, args, of_type, directives, description, join_fields }
    })
}

fn parse_input_field(
    input_field_node: async_graphql_parser::types::InputValueDefinition,
) -> Valid<InputFieldDefinition, String> {
    let name = input_field_node.name.to_string();
    let description = input_field_node.description.map(|d| d.to_string());
    let of_type = map_type(&input_field_node.ty.into_inner());

    let default_value = match input_field_node.default_value {
        Some(Positioned { pos, node: argument_value }) => {
            parse_argument(pos, argument_value).map(Some)
        }
        None => Valid::succeed(None),
    };

    let directives = Valid::from_iter(
        input_field_node.directives,
        |Positioned { pos: _, node: directive_node }| parse_directive(directive_node),
    );

    directives
        .zip(default_value)
        .map(|(directives, default_value)| {
            let join_fields: Vec<JoinField> =
                extract_join!(directives.into_iter(), "join__field", JoinField);

            InputFieldDefinition { name, of_type, default_value, description, join_fields }
        })
}

fn map_type(type_: &async_graphql_parser::types::Type) -> Type {
    match &type_.base {
        async_graphql_parser::types::BaseType::Named(name) => {
            Type::Named { name: name.to_string(), required: !type_.nullable }
        }
        async_graphql_parser::types::BaseType::List(inner_type) => Type::List {
            of_type: Box::new(map_type(inner_type.as_ref())),
            non_null: !type_.nullable,
        },
    }
}

fn parse_schema(
    schema_node: async_graphql_parser::types::SchemaDefinition,
) -> Valid<SchemaDefinition, String> {
    let query = if let Some(Positioned { pos: _, node: query_node }) = schema_node.query {
        Some(query_node.to_string())
    } else {
        None
    };

    let mutation = if let Some(Positioned { pos: _, node: mutation_node }) = schema_node.mutation {
        Some(mutation_node.to_string())
    } else {
        None
    };

    let subscription =
        if let Some(Positioned { pos: _, node: subscription_node }) = schema_node.subscription {
            Some(subscription_node.to_string())
        } else {
            None
        };

    let directives = Valid::from_iter(
        schema_node.directives,
        |Positioned { pos: _, node: directive_node }| parse_directive(directive_node),
    );

    directives.map(|directives| SchemaDefinition { query, mutation, subscription, directives })
}

fn parse_directive(
    directive_node: async_graphql_parser::types::ConstDirective,
) -> Valid<Directive, String> {
    let arguments = parse_arguments(directive_node.arguments);

    arguments.map(|arguments| Directive { name: directive_node.name.to_string(), arguments })
}

fn parse_arguments(
    arguments: Vec<(Positioned<Name>, Positioned<ConstValue>)>,
) -> Valid<serde_json::Value, String> {
    let map: Valid<BTreeMap<String, serde_json::Value>, String> = Valid::from_iter(
        arguments,
        |(Positioned { pos: _, node: argument_name }, Positioned { pos, node: argument_node })| {
            Valid::succeed(argument_name.to_string()).zip(parse_argument(pos, argument_node))
        },
    )
    .map(|arguments| {
        let mut data = BTreeMap::<String, serde_json::Value>::new();
        for (name, argument) in arguments {
            data.insert(name, argument);
        }
        data
    });
    map.map(|map| serde_json::Value::Object(map.into_iter().collect()))
}

fn parse_argument(
    pos: Pos,
    argument_node: async_graphql_value::ConstValue,
) -> Valid<serde_json::Value, String> {
    match argument_node.into_json() {
        Ok(value) => Valid::succeed(value),
        Err(_) => Valid::fail(format!(
            "Could not convert `ConstValue` to `Value` @{:}",
            pos
        )),
    }
}

#[cfg(test)]
mod tests {
    use valid::Validator;

    use super::*;

    #[test]
    fn test_parse() {
        let graphql = resource::resource_str!("../examples/router.graphql");
        let document = async_graphql_parser::parse_schema(graphql).unwrap();
        let blueprint = parse(document)
            .map(|b| serde_json::to_string_pretty(&b).unwrap())
            .to_result()
            .unwrap();
        insta::assert_snapshot!(blueprint);
    }
}
